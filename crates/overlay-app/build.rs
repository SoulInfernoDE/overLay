//! Embeds every `locales/<locale>/*.ftl` file, so the binary needs no catalog files at
//! runtime, then runs the Tauri build step.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::{env, fs};

fn main() {
    embed_locales();
    tauri_build::build();
}

fn embed_locales() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../locales");
    println!("cargo:rerun-if-changed={}", root.display());

    let mut entries = String::new();
    for locale_dir in sorted_entries(&root) {
        if !locale_dir.is_dir() {
            continue;
        }
        println!("cargo:rerun-if-changed={}", locale_dir.display());
        let locale = file_name(&locale_dir);
        for file in sorted_entries(&locale_dir) {
            if file.extension().is_none_or(|ext| ext != "ftl") {
                continue;
            }
            let path = fs::canonicalize(&file).expect("catalog path resolves");
            println!("cargo:rerun-if-changed={}", path.display());
            writeln!(
                entries,
                "    ({locale:?}, {name:?}, include_str!({path:?})),",
                name = file_name(&file),
            )
            .unwrap();
        }
    }

    let out = PathBuf::from(env::var("OUT_DIR").unwrap()).join("locales.rs");
    let code = format!("pub const CATALOGS: &[overlay_i18n::EmbeddedFile] = &[\n{entries}];\n");
    fs::write(out, code).expect("write embedded catalog list");
}

fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    paths
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}
