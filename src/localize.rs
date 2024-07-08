// SPDX-License-Identifier: GPL-3.0-only

use std::str::FromStr;

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DefaultLocalizer, LanguageLoader, Localizer,
};
use icu::collator::{Collator, CollatorOptions, Numeric};
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
    let collator = {
        let current = LANGUAGE_LOADER.current_language().to_string();

        let locale = DataLocale::from_str(&current).unwrap();
        let collator = Collator::try_new(&locale, options).unwrap();
        collator
    };

    collator
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
