use crate::{DEFAULT_CONFIG_FILE, JSON_CONFIG_FILE};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum OnionCryError {
    #[error(
        "could not find {DEFAULT_CONFIG_FILE} or {JSON_CONFIG_FILE}; pass --config <path> to use a different file"
    )]
    MissingDefaultConfig,
    #[error("could not read config {path}: {source}")]
    ReadConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("config already exists at {path}; pass --force to overwrite it")]
    ConfigAlreadyExists { path: PathBuf },
    #[error("could not write config {path}: {source}")]
    WriteConfig {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not write schema {path}: {source}")]
    WriteSchema {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not write baseline {path}: {source}")]
    WriteBaseline {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(
        "baseline file does not exist at {path}; pass --write-baseline to create it or --no-baseline to skip baseline consumption"
    )]
    MissingBaseline { path: PathBuf },
    #[error("could not read baseline {path}: {source}")]
    ReadBaseline {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not render schema: {source}")]
    RenderSchema {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not render baseline: {source}")]
    RenderBaseline {
        #[source]
        source: serde_json::Error,
    },
    #[error("could not parse baseline {path}: {source}")]
    ParseBaseline {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error(
        "unsupported baseline version {version} at {path}; expected version 1, or rerun --write-baseline to rewrite it"
    )]
    UnsupportedBaselineVersion { path: PathBuf, version: u8 },
    #[error("could not parse JSONC config {path}: {message}")]
    ParseConfig { path: PathBuf, message: String },
    #[error("project root does not exist: {path}")]
    MissingProjectRoot { path: PathBuf },
    #[error("invalid glob pattern {pattern:?}: {source}")]
    InvalidGlob {
        pattern: String,
        #[source]
        source: globset::Error,
    },
    #[error("could not read source file {path}: {source}")]
    ReadSource {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not parse source file {path}: {message}")]
    ParseSource { path: PathBuf, message: String },
    #[error("unknown rule {rule:?}; expected one of: {expected}")]
    UnknownRule { rule: String, expected: String },
    #[error("invalid value for rule {rule:?}: {message}")]
    InvalidRuleValue { rule: String, message: String },
    #[error(
        "rule {rule:?} is incompatible with architecture.mode {mode:?}; expected rules from {expected_family}"
    )]
    ArchitectureRuleModeMismatch {
        rule: String,
        mode: &'static str,
        expected_family: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, OnionCryError>;
