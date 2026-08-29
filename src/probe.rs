use std::io;
use std::process::Command;
use std::time::Instant;

#[derive(Debug)]
pub struct ProbeResult {
    pub code: String,
    pub milliseconds: u128,
}

pub fn probe(url: &str) -> io::Result<ProbeResult> {
    let started = Instant::now();
    let output_path = if cfg!(windows) { "NUL" } else { "/dev/null" };
    let output = Command::new("curl")
        .args([
            "--location",
            "--silent",
            "--show-error",
            "--max-time",
            "10",
            "--output",
            output_path,
            "--write-out",
            "%{http_code}",
            url,
        ])
        .output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(io::Error::other(if detail.is_empty() {
            format!("curl exited with {}", output.status)
        } else {
            detail
        }));
    }
    let code = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if code == "000" {
        return Err(io::Error::other("mirror did not return an HTTP status"));
    }
    Ok(ProbeResult {
        code,
        milliseconds: started.elapsed().as_millis(),
    })
}
