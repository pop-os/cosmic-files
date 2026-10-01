// SPDX-License-Identifier: GPL-3.0-only

use i18n_embed::fluent::{FluentLanguageLoader, fluent_language_loader};
use i18n_embed::{DefaultLocalizer, LanguageLoader, Localizer};
use icu::collator::options::CollatorOptions;
use icu::collator::preferences::CollationNumericOrdering;
use icu::collator::{Collator, CollatorBorrowed, CollatorPreferences};
use icu::datetime::options::TimePrecision;
use icu::datetime::preferences::HourCycle;
use icu::datetime::{DateTimeFormatter, DateTimeFormatterPreferences, fieldsets};
use icu::locale::Locale;
use icu::time::DateTime;
use jiff_icu::ConvertFrom;
use rust_embed::RustEmbed;
use std::fmt::{self, Display};
use std::sync::LazyLock;
use std::time::{Duration, SystemTime};

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

pub static LANGUAGE_SORTER: LazyLock<CollatorBorrowed> = LazyLock::new(|| {
    let create_collator = |locale: Locale| {
        let mut prefs = CollatorPreferences::from(locale);
        prefs.numeric_ordering = Some(CollationNumericOrdering::True);
        Collator::try_new(prefs, CollatorOptions::default()).ok()
    };

    Locale::try_from_str(&LANGUAGE_LOADER.current_language().to_string())
            .ok()
            .and_then(create_collator)
            .or_else(|| {
                Locale::try_from_str(&LANGUAGE_LOADER.fallback_language().to_string())
                    .ok()
                    .and_then(create_collator)
            })
            .unwrap_or_else(|| {
                let locale = Locale::try_from_str("en-US").expect("en-US is a valid BCP-47 tag");
                create_collator(locale)
                    .expect("Creating a collator from the system's current language, the fallback language, or American English should succeed")
            })
});

pub static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    for var in ["LC_TIME", "LC_ALL", "LANG"] {
        if let Ok(locale_str) = std::env::var(var) {
            let cleaned_locale = locale_str
                .split('.')
                .next()
                .unwrap_or(&locale_str)
                .replace('_', "-");

            if let Ok(locale) = Locale::try_from_str(&cleaned_locale) {
                return locale;
            }

            // Try language-only fallback (e.g., "en" from "en-US")
            if let Some(lang) = cleaned_locale.split('-').next()
                && let Ok(locale) = Locale::try_from_str(lang)
            {
                return locale;
            }
        }
    }
    log::warn!("No valid locale found in environment, using fallback");
    Locale::try_from_str("en-US").expect("Failed to parse fallback locale 'en-US'")
});

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

// Get the `Localizer` to be used for localizing this library.
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn localize() {
    let localizer = localizer();
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    if let Err(error) = localizer.select(&requested_languages) {
        eprintln!("Error while loading language for COSMIC Files {error}");
    }
}

pub(crate) fn date_time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::YMDT> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::YMDT::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

pub(crate) fn time_formatter(military_time: bool) -> DateTimeFormatter<fieldsets::T> {
    let mut prefs = DateTimeFormatterPreferences::from(LOCALE.clone());
    prefs.hour_cycle = Some(if military_time {
        HourCycle::H23
    } else {
        HourCycle::H12
    });

    let mut fs = fieldsets::T::medium();
    fs = fs.with_time_precision(TimePrecision::Minute);

    DateTimeFormatter::try_new(prefs, fs).expect("failed to create DateTimeFormatter")
}

pub(crate) struct FormatTime<'a> {
    pub time: SystemTime,
    pub date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    pub time_formatter: &'a DateTimeFormatter<fieldsets::T>,
}

impl<'a> FormatTime<'a> {
    pub(crate) fn from_secs(
        secs: i64,
        date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
        time_formatter: &'a DateTimeFormatter<fieldsets::T>,
    ) -> Option<Self> {
        // This looks convoluted because we need to ensure the units match up
        let secs: u64 = secs.try_into().ok()?;
        let now = SystemTime::now();
        let filetime_diff = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|from_epoch| from_epoch.as_secs())
            .ok()
            .and_then(|now_secs| now_secs.checked_sub(secs))
            .map(Duration::from_secs)?;
        now.checked_sub(filetime_diff).map(|time| Self {
            time,
            date_time_formatter,
            time_formatter,
        })
    }
}

impl Display for FormatTime<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Ok(zoned) = jiff::Zoned::try_from(self.time) else {
            return Ok(());
        };
        let now = jiff::Zoned::now();
        let icu_datetime = DateTime::convert_from(zoned.datetime());
        if zoned.date() == now.date() {
            f.write_str(fl!("today").as_str())?;
            f.write_str(", ")?;
            self.time_formatter.format(&icu_datetime).fmt(f)
        } else {
            self.date_time_formatter.format(&icu_datetime).fmt(f)
        }
    }
}

pub(crate) const fn format_time<'a>(
    time: SystemTime,
    date_time_formatter: &'a DateTimeFormatter<fieldsets::YMDT>,
    time_formatter: &'a DateTimeFormatter<fieldsets::T>,
) -> FormatTime<'a> {
    FormatTime {
        time,
        date_time_formatter,
        time_formatter,
    }
}
