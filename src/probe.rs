use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub code: String,
    pub state: String,
    pub detail: String,
    pub probe_url: String,
    pub milliseconds: u128,
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
        let path = env::var_os("LM_CACHE_FILE")
            .map(std::path::PathBuf::from)
            .or_else(|| dirs::cache_dir().map(|path| path.join("lazy-mirror/health.json")));
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

    pub fn get(&self, target: &str, url: &str) -> Option<ProbeResult> {
        let entry = self.entries.get(&cache_key(target, url))?;
        let now = unix_seconds();
        (now.saturating_sub(entry.checked_at) <= self.ttl_seconds).then(|| entry.result.clone())
    }

    pub fn put(&mut self, target: &str, url: &str, result: ProbeResult) {
        if self.path.is_some() {
            self.entries.insert(
                cache_key(target, url),
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
    probe_target("generic", url, 10, 0)
}

pub fn probe_target(
    target: &str,
    url: &str,
    timeout_seconds: u64,
    retries: u32,
) -> io::Result<ProbeResult> {
    let probe_url = target_url(target, url);
    let mut last_error = None;
    for _ in 0..=retries {
        match request(&probe_url, timeout_seconds) {
            Ok(result) => return Ok(result),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.unwrap_or_else(|| io::Error::other("mirror probe failed")))
}

fn request(url: &str, timeout_seconds: u64) -> io::Result<ProbeResult> {
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
        "%{http_code}".to_owned(),
    ];
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
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(io::Error::other(if detail.is_empty() {
            format!("curl exited with {}", output.status)
        } else {
            detail
        }));
    }
    let code = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if code == "000" || code.is_empty() {
        return Err(io::Error::other("mirror did not return an HTTP status"));
    }
    let state = classify(&code);
    Ok(ProbeResult {
        detail: state_detail(&state),
        state,
        probe_url: url.to_owned(),
        code,
        milliseconds: started.elapsed().as_millis(),
    })
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
    let url = url.split_once(',').map_or(url, |(url, _)| url);
    let url = url.trim_end_matches('/');
    let suffix = match target {
        "apt" => format!(
            "/dists/{}/Release",
            std::env::var("LM_APT_DISTRIBUTION").unwrap_or_else(|_| "stable".to_owned())
        ),
        "npm" | "pnpm" | "yarn" | "bun" => "/-/ping".to_owned(),
        "pip" | "uv" | "pdm" | "poetry" => "/simple/".to_owned(),
        "docker" | "containerd" | "podman" => "/v2/".to_owned(),
        "conda" => "/pkgs/main/repodata.json".to_owned(),
        "cran" => "/src/contrib/PACKAGES".to_owned(),
        "ros" => format!(
            "/dists/{}/Release",
            std::env::var("LM_ROS_DISTRIBUTION")
                .or_else(|_| std::env::var("ROS_DISTRO"))
                .unwrap_or_else(|_| "stable".to_owned())
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
    if suffix.is_empty() || url.ends_with(suffix.trim_end_matches('/')) {
        url.to_owned()
    } else {
        format!("{url}{suffix}")
    }
}

fn cache_key(target: &str, url: &str) -> String {
    format!("{target}\n{url}")
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
            let checked_at = value.get("checked_at")?.as_u64()?;
            let result = value.get("result")?;
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
            target_url("pip", "https://pypi.example/simple"),
            "https://pypi.example/simple"
        );
        assert_eq!(
            target_url("huggingface", "https://hf.example"),
            "https://hf.example/api/models?limit=1"
        );
    }

    #[test]
    fn status_classification_keeps_auth_reachable() {
        assert_eq!(classify("200"), "healthy");
        assert_eq!(classify("401"), "auth-required");
        assert_eq!(classify("429"), "rate-limited");
        assert_eq!(classify("404"), "unsupported");
    }
}
