//! Guards the bilingual rule: every shipped locale must define every English message.

use std::path::PathBuf;

use overlay_i18n::{Localizer, available_locales, check_completeness};

fn repo_locales() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../locales")
}

#[test]
fn all_catalogs_parse() {
    for locale in available_locales(&repo_locales()).unwrap() {
        Localizer::load(&repo_locales(), &[locale]).unwrap();
    }
}

#[test]
fn all_locales_are_complete() {
    let report = check_completeness(&repo_locales()).unwrap();
    assert!(report.is_empty(), "missing translations: {report:#?}");
}
