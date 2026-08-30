use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "lower")]
pub enum Scope {
    Project,
    User,
    System,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    settings: Settings,
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub timeout_seconds: u64,
    pub retries: u32,
    pub cache_ttl_seconds: u64,
    pub parallelism: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            retries: 1,
            cache_ttl_seconds: 0,
            parallelism: 4,
        }
    }
}

impl Config {
    pub fn settings(&self) -> Settings {
        self.settings
    }
}

pub(crate) fn is_url(value: &str) -> bool {
    let authority = value
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
        .unwrap_or_default();
    (value.starts_with("http://") || value.starts_with("https://"))
        && !authority.is_empty()
        && !authority.contains('@')
        && !value.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || matches!(character, '"' | '\'' | '\\' | '$' | '`')
        })
}

pub fn redact_selection(value: &str) -> String {
    let url = value.strip_prefix("sparse+").unwrap_or(value);
    if url.starts_with("http://") || url.starts_with("https://") {
        redact_url(value)
    } else {
        value.to_owned()
    }
}

fn redact_url(value: &str) -> String {
    let Some(scheme) = value.find("://") else {
        return value.to_owned();
    };
    let authority_start = scheme + 3;
    let authority_end = value[authority_start..]
        .find(['/', '?', '#'])
        .map_or(value.len(), |offset| authority_start + offset);
    let authority = &value[authority_start..authority_end];
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let suffix = &value[authority_end..];
    let suffix = suffix
        .find(['?', '#'])
        .map_or(suffix, |offset| &suffix[..offset]);
    format!("{}://{}{}", &value[..scheme], authority, suffix)
}
