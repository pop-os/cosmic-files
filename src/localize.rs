// SPDX-License-Identifier: GPL-3.0-only

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use icu::locid::Locale;
use icu_collator::{Collator, CollatorOptions, Numeric};
use icu_provider::DataLocale;
use rust_embed::RustEmbed;
use std::str::FromStr;
use std::sync::LazyLock;

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

pub static LANGUAGE_SORTER: LazyLock<Collator> = LazyLock::new(|| {
    let mut options = CollatorOptions::new();
    options.numeric = Some(Numeric::On);

    DataLocale::from_str(&LANGUAGE_LOADER.current_language().to_string())
        .or_else(|_| DataLocale::from_str(&LANGUAGE_LOADER.fallback_language().to_string()))
        .ok()
        .and_then(|locale| Collator::try_new(&locale, options).ok())
        .or_else(|| {
            let locale = DataLocale::from_str("en-US").expect("en-US is a valid BCP-47 tag");
            Collator::try_new(&locale, options).ok()
        })
        .expect("Creating a collator from the system's current language, the fallback language, or American English should succeed")
});

pub static LOCALE: LazyLock<Locale> = LazyLock::new(|| {
    fn get_local() -> Result<Locale, Box<dyn std::error::Error>> {
        let locale = std::env::var("LC_TIME").or_else(|_| std::env::var("LANG"))?;

        let locale = locale
            .split('.')
            .next()
            .ok_or(format!("Can't split the locale {locale}"))?;

        let locale = Locale::from_str(locale).map_err(|e| format!("{e:?}"))?;

        Ok(locale)
    }

    match get_local() {
        Ok(locale) => locale,

        Err(e) => {
            log::error!("can't get locale {e}");

            Locale::default()
        }
    }
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
        eprintln!("Error while loading language for App List {}", error);
    }
}
