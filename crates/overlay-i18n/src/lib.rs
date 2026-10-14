//! Fluent-based localization for overLay.
//!
//! Catalogs live in `<root>/<locale>/*.ftl`. English (`en`) is the base locale and the
//! last entry of every fallback chain, so a message missing in another locale still
//! renders in English instead of showing its raw ID.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use fluent_bundle::FluentResource;
use fluent_bundle::concurrent::FluentBundle;
pub use fluent_bundle::{FluentArgs, FluentValue};
use fluent_langneg::{NegotiationStrategy, negotiate_languages};
use fluent_syntax::ast;
use unic_langid::LanguageIdentifier;

/// Base and fallback locale of every catalog.
pub const FALLBACK_LOCALE: &str = "en";

#[derive(Debug, thiserror::Error)]
pub enum I18nError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid locale directory name {0:?}")]
    InvalidLocale(String),
    #[error("syntax errors in {path}: {details}")]
    Syntax { path: PathBuf, details: String },
    #[error("duplicate message IDs in {path}: {ids}")]
    Duplicate { path: PathBuf, ids: String },
    #[error("base locale {FALLBACK_LOCALE:?} is missing in {0}")]
    MissingBase(PathBuf),
}

/// Messages that exist in the base locale but not in `locale`.
#[derive(Debug, PartialEq, Eq)]
pub struct MissingMessages {
    pub locale: LanguageIdentifier,
    pub ids: Vec<String>,
}

/// A catalog file compiled into a binary: `(locale, file name, contents)`.
pub type EmbeddedFile = (&'static str, &'static str, &'static str);

/// Formats messages along a negotiated fallback chain of locales.
pub struct Localizer {
    chain: Vec<(LanguageIdentifier, FluentBundle<FluentResource>)>,
}

impl Localizer {
    /// Loads the catalogs under `root` and picks the best matches for `requested`,
    /// always ending the chain with the base locale.
    pub fn load(root: &Path, requested: &[LanguageIdentifier]) -> Result<Self, I18nError> {
        let available = available_locales(root)?;
        Self::build(root, &available, requested, |locale| {
            read_catalog(root, locale)
        })
    }

    /// Like [`Localizer::load`], for catalogs embedded with `include_str!`.
    pub fn from_embedded(
        files: &[EmbeddedFile],
        requested: &[LanguageIdentifier],
    ) -> Result<Self, I18nError> {
        let mut available = Vec::new();
        for (locale, _, _) in files {
            let id = locale
                .parse::<LanguageIdentifier>()
                .map_err(|_| I18nError::InvalidLocale((*locale).to_owned()))?;
            if !available.contains(&id) {
                available.push(id);
            }
        }
        Self::build(Path::new("<embedded>"), &available, requested, |locale| {
            let name = locale.to_string();
            Ok(files
                .iter()
                .filter(|(l, _, _)| *l == name)
                .map(|(l, file, source)| {
                    (PathBuf::from(format!("{l}/{file}")), (*source).to_owned())
                })
                .collect())
        })
    }

    fn build(
        origin: &Path,
        available: &[LanguageIdentifier],
        requested: &[LanguageIdentifier],
        read: impl Fn(&LanguageIdentifier) -> Result<Vec<(PathBuf, String)>, I18nError>,
    ) -> Result<Self, I18nError> {
        let base = base_locale();
        if !available.contains(&base) {
            return Err(I18nError::MissingBase(origin.to_path_buf()));
        }

        let mut order: Vec<LanguageIdentifier> = negotiate_languages(
            requested,
            available,
            Some(&base),
            NegotiationStrategy::Filtering,
        )
        .into_iter()
        .cloned()
        .collect();
        order.retain(|l| *l != base);
        order.push(base);

        let chain = order
            .into_iter()
            .map(|locale| {
                let bundle = build_bundle(&locale, read(&locale)?)?;
                Ok((locale, bundle))
            })
            .collect::<Result<_, I18nError>>()?;
        Ok(Self { chain })
    }

    /// The resolved fallback chain, most preferred first.
    pub fn locales(&self) -> impl Iterator<Item = &LanguageIdentifier> {
        self.chain.iter().map(|(l, _)| l)
    }

    /// Formats `id` with the first locale that defines it, or returns `id` itself
    /// when no catalog has it.
    pub fn format(&self, id: &str, args: Option<&FluentArgs>) -> String {
        for (_, bundle) in &self.chain {
            let Some(pattern) = bundle.get_message(id).and_then(|m| m.value()) else {
                continue;
            };
            let mut errors = Vec::new();
            return bundle
                .format_pattern(pattern, args, &mut errors)
                .into_owned();
        }
        id.to_owned()
    }
}

/// Lists the locale directories found under `root`, sorted.
pub fn available_locales(root: &Path) -> Result<Vec<LanguageIdentifier>, I18nError> {
    let entries = fs::read_dir(root).map_err(|source| I18nError::Io {
        path: root.to_path_buf(),
        source,
    })?;
    let mut locales = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| I18nError::Io {
            path: root.to_path_buf(),
            source,
        })?;
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let locale = name
            .parse::<LanguageIdentifier>()
            .map_err(|_| I18nError::InvalidLocale(name.clone()))?;
        locales.push(locale);
    }
    locales.sort_by_key(|l| l.to_string());
    Ok(locales)
}

/// Collects all message and term IDs (terms keep their leading `-`) of one locale.
pub fn message_ids(
    root: &Path,
    locale: &LanguageIdentifier,
) -> Result<BTreeSet<String>, I18nError> {
    let mut ids = BTreeSet::new();
    for (path, source) in read_catalog(root, locale)? {
        let resource = parse(&path, source)?;
        for entry in resource.entries() {
            match entry {
                ast::Entry::Message(m) => ids.insert(m.id.name.to_owned()),
                ast::Entry::Term(t) => ids.insert(format!("-{}", t.id.name)),
                _ => continue,
            };
        }
    }
    Ok(ids)
}

/// Compares every locale against the base locale and reports missing IDs.
pub fn check_completeness(root: &Path) -> Result<Vec<MissingMessages>, I18nError> {
    let base = base_locale();
    let base_ids = message_ids(root, &base)?;
    let mut report = Vec::new();
    for locale in available_locales(root)? {
        if locale == base {
            continue;
        }
        let ids = message_ids(root, &locale)?;
        let missing: Vec<String> = base_ids.difference(&ids).cloned().collect();
        if !missing.is_empty() {
            report.push(MissingMessages {
                locale,
                ids: missing,
            });
        }
    }
    Ok(report)
}

fn base_locale() -> LanguageIdentifier {
    FALLBACK_LOCALE
        .parse()
        .expect("base locale is a valid identifier")
}

fn read_catalog(
    root: &Path,
    locale: &LanguageIdentifier,
) -> Result<Vec<(PathBuf, String)>, I18nError> {
    let dir = root.join(locale.to_string());
    let entries = fs::read_dir(&dir).map_err(|source| I18nError::Io {
        path: dir.clone(),
        source,
    })?;
    let mut paths: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "ftl"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let source = fs::read_to_string(&path).map_err(|source| I18nError::Io {
                path: path.clone(),
                source,
            })?;
            Ok((path, source))
        })
        .collect()
}

fn parse(path: &Path, source: String) -> Result<FluentResource, I18nError> {
    FluentResource::try_new(source).map_err(|(_, errors)| I18nError::Syntax {
        path: path.to_path_buf(),
        details: errors
            .iter()
            .map(|e| format!("{e:?}"))
            .collect::<Vec<_>>()
            .join("; "),
    })
}

fn build_bundle(
    locale: &LanguageIdentifier,
    files: Vec<(PathBuf, String)>,
) -> Result<FluentBundle<FluentResource>, I18nError> {
    let mut bundle = FluentBundle::new_concurrent(vec![locale.clone()]);
    for (path, source) in files {
        let resource = parse(&path, source)?;
        bundle
            .add_resource(resource)
            .map_err(|errors| I18nError::Duplicate {
                path: path.clone(),
                ids: errors
                    .iter()
                    .map(|e| format!("{e:?}"))
                    .collect::<Vec<_>>()
                    .join("; "),
            })?;
    }
    Ok(bundle)
}
