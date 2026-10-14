//! Localizer for host-side text (tray menu), built from the catalogs embedded at build time.

use overlay_i18n::Localizer;
use unic_langid::LanguageIdentifier;

include!(concat!(env!("OUT_DIR"), "/locales.rs"));

/// `language` is the user's choice; `None` follows the system languages.
pub fn requested_languages(language: Option<&str>) -> Vec<LanguageIdentifier> {
    match language {
        Some(language) => language.parse().into_iter().collect(),
        None => sys_locale::get_locales()
            .filter_map(|l| l.parse().ok())
            .collect(),
    }
}

pub fn localizer(language: Option<&str>) -> Localizer {
    // The catalogs are checked by overlay-i18n's repo_catalogs test.
    Localizer::from_embedded(CATALOGS, &requested_languages(language))
        .expect("embedded catalogs are valid")
}
