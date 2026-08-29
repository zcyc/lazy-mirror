use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum IpVersion {
    #[default]
    Any,
    V4,
    V6,
}

#[derive(Debug, Clone)]
pub struct ProbeMetrics {
    pub remote_ip: Option<String>,
    pub content_type: Option<String>,
    pub dns_milliseconds: u128,
    pub connect_milliseconds: u128,
    pub tls_milliseconds: u128,
    pub ttfb_milliseconds: u128,
}

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub code: String,
    pub state: String,
    pub detail: String,
    pub probe_url: String,
    pub milliseconds: u128,
    pub metrics: ProbeMetrics,
}

#[derive(Debug)]
struct CacheEntry {
    checked_at: u64,
    result: ProbeResult,
}

pub struct HealthCache {
    path: Option<std::path::PathBuf>,
    ttl_seconds: u64,
    entries: BTreeMap<String, CacheEntry>,
}

impl HealthCache {
    pub fn load(ttl_seconds: u64) -> io::Result<Self> {
        if ttl_seconds == 0 {
            return Ok(Self {
                path: None,
                ttl_seconds,
                entries: BTreeMap::new(),
            });
        }
        let path = if let Some(path) = env::var_os("LM_CACHE_FILE") {
            if path.is_empty() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "LM_CACHE_FILE cannot be empty",
                ));
            }
            Some(path.into())
        } else {
            dirs::cache_dir().map(|path| path.join("lazy-mirror/health.json"))
        };
        let Some(path) = path else {
            return Ok(Self {
                path: None,
                ttl_seconds,
                entries: BTreeMap::new(),
            });
        };
        let entries = match std::fs::read_to_string(&path) {
            Ok(content) => parse_cache(&content),
            Err(error) if error.kind() == io::ErrorKind::NotFound => BTreeMap::new(),
            Err(error) => return Err(error),
        };
        Ok(Self {
            path: Some(path),
            ttl_seconds,
            entries,
        })
    }

    pub fn get(&self, target: &str, url: &str, ip_version: IpVersion) -> Option<ProbeResult> {
        if !cacheable_url(url) {
            return None;
        }
        let entry = self.entries.get(&cache_key(target, url, ip_version))?;
        let now = unix_seconds();
        (now.saturating_sub(entry.checked_at) <= self.ttl_seconds).then(|| entry.result.clone())
    }

    pub fn put(&mut self, target: &str, url: &str, ip_version: IpVersion, result: ProbeResult) {
        if self.path.is_some() && cacheable_url(url) {
            self.entries.insert(
                cache_key(target, url, ip_version),
                CacheEntry {
                    checked_at: unix_seconds(),
                    result,
                },
            );
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        let mut object = serde_json::Map::new();
        for (key, entry) in &self.entries {
            object.insert(
                key.clone(),
                serde_json::json!({
                    "checked_at": entry.checked_at,
                    "result": {
                        "code": entry.result.code.clone(),
                        "state": entry.result.state.clone(),
                        "detail": entry.result.detail.clone(),
                        "probe_url": entry.result.probe_url.clone(),
                        "milliseconds": entry.result.milliseconds as u64,
                        "metrics": {
                            "remote_ip": entry.result.metrics.remote_ip.clone(),
                            "content_type": entry.result.metrics.content_type.clone(),
                            "dns_milliseconds": entry.result.metrics.dns_milliseconds as u64,
                            "connect_milliseconds": entry.result.metrics.connect_milliseconds as u64,
                            "tls_milliseconds": entry.result.metrics.tls_milliseconds as u64,
                            "ttfb_milliseconds": entry.result.metrics.ttfb_milliseconds as u64,
                        },
                    }
                }),
            );
        }
        let content = serde_json::to_string_pretty(&object)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
            + "\n";
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let _lock = crate::lock(path)?;
        crate::atomic_write(path, &content)
    }
}

pub fn probe(url: &str) -> io::Result<ProbeResult> {
    probe_target("generic", url, 10, 0, IpVersion::Any)
}

pub fn probe_target(
    target: &str,
    url: &str,
    timeout_seconds: u64,
    retries: u32,
    ip_version: IpVersion,
) -> io::Result<ProbeResult> {
    let probe_url = target_url(target, url);
    let mut last_error = None;
    for _ in 0..=retries {
        match request(target, &probe_url, timeout_seconds, ip_version) {
            Ok(result) => return Ok(result),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("mirror probe failed")))
}

fn request(
    target: &str,
    url: &str,
    timeout_seconds: u64,
    ip_version: IpVersion,
) -> io::Result<ProbeResult> {
    let started = Instant::now();
    let output_path = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let timeout = timeout_seconds.max(1).to_string();
    let mut args = vec![
        "--location".to_owned(),
        "--silent".to_owned(),
        "--show-error".to_owned(),
        "--connect-timeout".to_owned(),
        timeout.clone(),
        "--max-time".to_owned(),
        timeout,
        "--user-agent".to_owned(),
        "lazy-mirror/0.1 (+mirror-check)".to_owned(),
        "--output".to_owned(),
        output_path.to_owned(),
        "--write-out".to_owned(),
        "%{http_code}\t%{content_type}\t%{remote_ip}\t%{time_namelookup}\t%{time_connect}\t%{time_appconnect}\t%{time_starttransfer}".to_owned(),
    ];
    match ip_version {
        IpVersion::Any => {}
        IpVersion::V4 => args.push("--ipv4".to_owned()),
        IpVersion::V6 => args.push("--ipv6".to_owned()),
    }
    args.push(url.to_owned());
    let curl_config = curl_auth_config();
    if curl_config.is_some() {
        args.extend(["--config".to_owned(), "-".to_owned()]);
    }
    let output = if let Some(curl_config) = curl_config {
        let mut child = Command::new("curl")
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(curl_config.as_bytes())?;
        }
        child.wait_with_output()?
    } else {
        Command::new("curl").args(&args).output()?
    };
    if !output.status.success() {
        let detail = redact_probe_text(String::from_utf8_lossy(&output.stderr).trim());
        return Err(io::Error::other(if detail.is_empty() {
            format!("curl exited with {}", output.status)
        } else {
            detail
        }));
    }
    let raw_metrics = parse_curl_metrics(String::from_utf8_lossy(&output.stdout).trim())?;
    if raw_metrics.code == "000" || raw_metrics.code.is_empty() {
        return Err(io::Error::other("mirror did not return an HTTP status"));
    }
    let state = classify_protocol_response(
        target,
        &raw_metrics.code,
        raw_metrics.content_type.as_deref(),
    );
    let detail = protocol_detail(&state, raw_metrics.content_type.as_deref());
    let code = raw_metrics.code.clone();
    let metrics = raw_metrics.into_public();
    Ok(ProbeResult {
        detail,
        state,
        probe_url: url.to_owned(),
        code,
        milliseconds: started.elapsed().as_millis(),
        metrics,
    })
}

fn parse_curl_metrics(output: &str) -> io::Result<ProbeResultMetrics> {
    let mut fields = output.split('\t');
    let code = fields.next().unwrap_or_default().to_owned();
    let content_type = non_empty(fields.next().unwrap_or_default());
    let remote_ip = non_empty(fields.next().unwrap_or_default());
    let dns_milliseconds = curl_seconds_to_milliseconds(fields.next().unwrap_or_default())?;
    let connect_milliseconds = curl_seconds_to_milliseconds(fields.next().unwrap_or_default())?;
    let tls_milliseconds = curl_seconds_to_milliseconds(fields.next().unwrap_or_default())?;
    let ttfb_milliseconds = curl_seconds_to_milliseconds(fields.next().unwrap_or_default())?;
    if fields.next().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "curl returned malformed probe metrics",
        ));
    }
    Ok(ProbeResultMetrics {
        code,
        content_type,
        remote_ip,
        dns_milliseconds,
        connect_milliseconds,
        tls_milliseconds,
        ttfb_milliseconds,
    })
}

struct ProbeResultMetrics {
    code: String,
    content_type: Option<String>,
    remote_ip: Option<String>,
    dns_milliseconds: u128,
    connect_milliseconds: u128,
    tls_milliseconds: u128,
    ttfb_milliseconds: u128,
}

impl ProbeResultMetrics {
    fn into_public(self) -> ProbeMetrics {
        ProbeMetrics {
            remote_ip: self.remote_ip,
            content_type: self.content_type,
            dns_milliseconds: self.dns_milliseconds,
            connect_milliseconds: self.connect_milliseconds,
            tls_milliseconds: self.tls_milliseconds,
            ttfb_milliseconds: self.ttfb_milliseconds,
        }
    }
}

fn non_empty(value: &str) -> Option<String> {
    (!value.is_empty()).then(|| value.to_owned())
}

fn curl_seconds_to_milliseconds(value: &str) -> io::Result<u128> {
    let value = value.parse::<f64>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "curl returned malformed probe timing",
        )
    })?;
    if !value.is_finite() || value < 0.0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "curl returned invalid probe timing",
        ));
    }
    Ok((value * 1000.0).round() as u128)
}

fn classify_protocol_response(target: &str, code: &str, content_type: Option<&str>) -> String {
    let state = classify(code);
    if state == "healthy"
        && matches!(
            target,
            "npm" | "pnpm" | "yarn" | "bun" | "conda" | "huggingface" | "nuget"
        )
        && !content_type.is_some_and(|value| value.to_ascii_lowercase().contains("json"))
    {
        "invalid-response".to_owned()
    } else {
        state
    }
}

fn protocol_detail(state: &str, content_type: Option<&str>) -> String {
    if state == "invalid-response" {
        return format!(
            "endpoint returned an unexpected content type: {}",
            content_type.unwrap_or("missing")
        );
    }
    state_detail(state)
}

fn curl_auth_config() -> Option<String> {
    let mut config = String::new();
    if let Some(token) = env::var_os("LM_MIRROR_TOKEN") {
        config.push_str("header = ");
        config.push_str(&curl_config_value(&format!(
            "Authorization: Bearer {}",
            token.to_string_lossy()
        )));
        config.push('\n');
    }
    if let (Some(user), Some(password)) = (
        env::var_os("LM_MIRROR_USERNAME"),
        env::var_os("LM_MIRROR_PASSWORD"),
    ) {
        config.push_str("user = ");
        config.push_str(&curl_config_value(&format!(
            "{}:{}",
            user.to_string_lossy(),
            password.to_string_lossy()
        )));
        config.push('\n');
    }
    (!config.is_empty()).then_some(config)
}

fn curl_config_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');
    for character in value.chars() {
        match character {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            character => escaped.push(character),
        }
    }
    escaped.push('"');
    escaped
}

fn classify(code: &str) -> String {
    match code.parse::<u16>() {
        Ok(200..=399) => "healthy",
        Ok(401 | 403) => "auth-required",
        Ok(404 | 410) => "unsupported",
        Ok(429) => "rate-limited",
        Ok(400..=499) => "unavailable",
        Ok(500..=599) => "unavailable",
        _ => "unknown",
    }
    .to_owned()
}

fn state_detail(state: &str) -> String {
    match state {
        "healthy" => "HTTP endpoint responded successfully".to_owned(),
        "auth-required" => "endpoint is reachable but requires authentication".to_owned(),
        "rate-limited" => "endpoint is reachable but rate limited".to_owned(),
        "unsupported" => "endpoint is reachable but probe path is unsupported".to_owned(),
        "unavailable" => "endpoint returned an error status".to_owned(),
        _ => "endpoint returned an unrecognized status".to_owned(),
    }
}

fn target_url(target: &str, url: &str) -> String {
    let url = url.strip_prefix("sparse+").unwrap_or(url);
    let url = if target == "go" {
        url.split_once(',').map_or(url, |(url, _)| url)
    } else {
        url
    };
    let url = url.trim_end_matches('/');
    let suffix = match target {
        "apt" => format!("/dists/{}/Release", crate::platform::apt_distribution()),
        "npm" | "pnpm" | "yarn" | "bun" => "/-/ping".to_owned(),
        "pip" | "uv" | "pdm" | "poetry" => "/simple/".to_owned(),
        "docker" | "buildkit" | "containerd" | "podman" => "/v2/".to_owned(),
        "conda" => "/pkgs/main/repodata.json".to_owned(),
        "cran" => "/src/contrib/PACKAGES".to_owned(),
        "ros" => format!(
            "/dists/{}/Release",
            std::env::var("LM_ROS_DISTRIBUTION")
                .or_else(|_| std::env::var("ROS_DISTRO"))
                .unwrap_or_else(|_| crate::platform::apt_distribution())
        ),
        "huggingface" => "/api/models?limit=1".to_owned(),
        "nuget" => "/v3/index.json".to_owned(),
        "apk" => "/latest-stable/main/x86_64/APKINDEX.tar.gz".to_owned(),
        "rustup" => "/dist/channel-rust-stable.toml".to_owned(),
        "cpan" => "/modules/02packages.details.txt.gz".to_owned(),
        "luarocks" => "/manifest".to_owned(),
        "hackage" | "cabal" | "stack" => "/01-index.tar.gz".to_owned(),
        "flathub" => "/summary".to_owned(),
        _ => String::new(),
    };
    let query_start = url.find(['?', '#']).unwrap_or(url.len());
    let (base, query) = url.split_at(query_start);
    let base = base.trim_end_matches('/');
    let suffix_query_start = suffix.find(['?', '#']).unwrap_or(suffix.len());
    let (suffix_path, suffix_query) = suffix.split_at(suffix_query_start);
    let path = if suffix_path.is_empty() || base.ends_with(suffix_path.trim_end_matches('/')) {
        base.to_owned()
    } else {
        format!("{base}{suffix_path}")
    };
    let query = match (suffix_query, query) {
        ("", query) => query.to_owned(),
        (suffix_query, "") => suffix_query.to_owned(),
        (suffix_query, query) if suffix_query.starts_with('?') && query.starts_with('?') => {
            format!("{suffix_query}&{}", &query[1..])
        }
        (suffix_query, query) => format!("{suffix_query}{query}"),
    };
    format!("{path}{query}")
}

fn cache_key(target: &str, url: &str, ip_version: IpVersion) -> String {
    let ip_version = match ip_version {
        IpVersion::Any => "any",
        IpVersion::V4 => "ipv4",
        IpVersion::V6 => "ipv6",
    };
    format!("{target}\n{ip_version}\n{url}")
}

fn cacheable_url(url: &str) -> bool {
    !url.contains(['?', '#'])
}

fn redact_probe_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(redact_probe_url)
        .collect::<Vec<_>>()
        .join(" ")
}

fn redact_probe_url(value: &str) -> String {
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

fn parse_cache(content: &str) -> BTreeMap<String, CacheEntry> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(content) else {
        return BTreeMap::new();
    };
    value
        .as_object()
        .into_iter()
        .flat_map(|entries| entries.iter())
        .filter_map(|(key, value)| {
            let mut parts = key.splitn(3, '\n');
            parts.next()?;
            match parts.next()? {
                "any" | "ipv4" | "ipv6" => {}
                _ => return None,
            }
            let source_url = parts.next()?;
            if !cacheable_url(source_url) {
                return None;
            }
            let checked_at = value.get("checked_at")?.as_u64()?;
            let result = value.get("result")?;
            let metrics = result.get("metrics")?;
            Some((
                key.clone(),
                CacheEntry {
                    checked_at,
                    result: ProbeResult {
                        code: result.get("code")?.as_str()?.to_owned(),
                        state: result.get("state")?.as_str()?.to_owned(),
                        detail: result.get("detail")?.as_str()?.to_owned(),
                        probe_url: result.get("probe_url")?.as_str()?.to_owned(),
                        milliseconds: result.get("milliseconds")?.as_u64()? as u128,
                        metrics: ProbeMetrics {
                            remote_ip: metrics.get("remote_ip")?.as_str().map(str::to_owned),
                            content_type: metrics.get("content_type")?.as_str().map(str::to_owned),
                            dns_milliseconds: metrics.get("dns_milliseconds")?.as_u64()? as u128,
                            connect_milliseconds: metrics.get("connect_milliseconds")?.as_u64()?
                                as u128,
                            tls_milliseconds: metrics.get("tls_milliseconds")?.as_u64()? as u128,
                            ttfb_milliseconds: metrics.get("ttfb_milliseconds")?.as_u64()? as u128,
                        },
                    },
                },
            ))
        })
        .collect()
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_probe_uses_package_protocol_paths() {
        assert_eq!(
            target_url("docker", "https://registry.example"),
            "https://registry.example/v2/"
        );
        assert_eq!(
            target_url("buildkit", "https://registry.example"),
            "https://registry.example/v2/"
        );
        assert_eq!(
            target_url("pip", "https://pypi.example/simple"),
            "https://pypi.example/simple"
        );
        assert_eq!(
            target_url("huggingface", "https://hf.example"),
            "https://hf.example/api/models?limit=1"
        );
        assert_eq!(
            target_url("pip", "https://pypi.example/simple?token=secret"),
            "https://pypi.example/simple?token=secret"
        );
        assert_eq!(
            target_url("huggingface", "https://hf.example?token=secret"),
            "https://hf.example/api/models?limit=1&token=secret"
        );
        assert_eq!(
            target_url("huggingface", "https://hf.example?token=a,b"),
            "https://hf.example/api/models?limit=1&token=a,b"
        );
    }

    #[test]
    fn status_classification_keeps_auth_reachable() {
        assert_eq!(classify("200"), "healthy");
        assert_eq!(classify("401"), "auth-required");
        assert_eq!(classify("429"), "rate-limited");
        assert_eq!(classify("404"), "unsupported");
        assert_eq!(
            classify_protocol_response("huggingface", "200", Some("text/html")),
            "invalid-response"
        );
        assert_eq!(
            classify_protocol_response("huggingface", "200", Some("application/json")),
            "healthy"
        );
    }

    #[test]
    fn curl_metrics_are_parsed_without_losing_empty_fields() {
        let metrics =
            parse_curl_metrics("200\tapplication/json\t192.0.2.1\t0.001\t0.002\t0.003\t0.004")
                .unwrap();
        assert_eq!(metrics.code, "200");
        assert_eq!(metrics.content_type.as_deref(), Some("application/json"));
        assert_eq!(metrics.remote_ip.as_deref(), Some("192.0.2.1"));
        assert_eq!(metrics.dns_milliseconds, 1);
        assert_eq!(metrics.tls_milliseconds, 3);
    }

    #[test]
    fn curl_auth_values_are_escaped_for_stdin_config() {
        let value = curl_config_value("user\"\\\n");
        assert_eq!(value, "\"user\\\"\\\\\\n\"");
    }

    #[test]
    fn query_urls_are_not_written_to_health_cache() {
        assert!(!cacheable_url("https://mirror.example/simple?token=secret"));
        assert!(cacheable_url("https://mirror.example/simple"));
    }

    #[test]
    fn probe_cache_keys_are_separate_by_ip_version() {
        assert_ne!(
            cache_key("pip", "https://mirror.example/simple", IpVersion::V4),
            cache_key("pip", "https://mirror.example/simple", IpVersion::V6)
        );
    }

    #[test]
    fn probe_errors_redact_credentials_and_queries() {
        assert_eq!(
            redact_probe_text("curl: https://user:secret@example.com/simple?token=secret failed"),
            "curl: https://example.com/simple failed"
        );
    }

    #[test]
    fn old_query_cache_entries_are_discarded() {
        let content = serde_json::json!({
            "pip\nany\nhttps://example.com/simple?token=secret": {
                "checked_at": 1,
                "result": {
                    "code": "200",
                    "state": "healthy",
                    "detail": "ok",
                    "probe_url": "https://example.com/simple?token=secret",
                    "milliseconds": 1
                }
            }
        })
        .to_string();
        assert!(parse_cache(&content).is_empty());
    }
}
