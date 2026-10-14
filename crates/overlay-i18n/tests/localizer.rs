use std::path::PathBuf;

use overlay_i18n::{Localizer, MissingMessages, check_completeness};
use unic_langid::langid;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/locales")
}

#[test]
fn prefers_requested_locale() {
    let l10n = Localizer::load(&fixtures(), &[langid!("de-DE")]).unwrap();
    assert_eq!(l10n.format("greeting", None), "Hallo");
}

#[test]
fn falls_back_to_english() {
    let l10n = Localizer::load(&fixtures(), &[langid!("de")]).unwrap();
    assert_eq!(l10n.format("only-english", None), "Only in English");
    assert_eq!(l10n.locales().count(), 2);
}

#[test]
fn unknown_locale_uses_english() {
    let l10n = Localizer::load(&fixtures(), &[langid!("fr")]).unwrap();
    assert_eq!(l10n.format("greeting", None), "Hello");
}

#[test]
fn unknown_id_returns_id() {
    let l10n = Localizer::load(&fixtures(), &[]).unwrap();
    assert_eq!(l10n.format("does-not-exist", None), "does-not-exist");
}

#[test]
fn reports_missing_messages() {
    let report = check_completeness(&fixtures()).unwrap();
    assert_eq!(
        report,
        vec![MissingMessages {
            locale: langid!("de"),
            ids: vec!["only-english".into()],
        }]
    );
}

#[test]
fn embedded_catalogs_fall_back_to_english() {
    let files = [
        (
            "en",
            "main.ftl",
            "greeting = Hello\nonly-english = Only in English\n",
        ),
        ("de", "main.ftl", "greeting = Hallo\n"),
    ];
    let l10n = overlay_i18n::Localizer::from_embedded(&files, &[langid!("de-AT")]).unwrap();
    assert_eq!(l10n.format("greeting", None), "Hallo");
    assert_eq!(l10n.format("only-english", None), "Only in English");
}

#[test]
fn embedded_catalogs_need_english() {
    let files = [("de", "main.ftl", "greeting = Hallo\n")];
    assert!(overlay_i18n::Localizer::from_embedded(&files, &[]).is_err());
}
