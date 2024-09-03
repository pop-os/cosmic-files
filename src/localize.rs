// SPDX-License-Identifier: GPL-3.0-only

use std::str::FromStr;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use icu_collator::{Collator, CollatorOptions, Numeric};
use icu_provider::DataLocale;
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: Lazy<FluentLanguageLoader> = Lazy::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

pub static LANGUAGE_SORTER: Lazy<Collator> = Lazy::new(|| {
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

pub static LANGUAGE_CHRONO: Lazy<chrono::Locale> = Lazy::new(|| {
    std::env::var("LC_TIME")
        .or_else(|_| std::env::var("LANG"))
        .ok()
        .and_then(|locale_full| {
            // Split LANG because it may be set to a locale such as en_US.UTF8
            locale_full
                .split('.')
                .next()
                .and_then(|locale| chrono::Locale::from_str(locale).ok())
        })
        .unwrap_or(chrono::Locale::en_US)
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
