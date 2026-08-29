use std::process::Command;

#[test]
fn list_json_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["list", "docker", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "lm/v1");
    assert_eq!(value["target"], "docker");
    assert_eq!(value["mirrors"][0]["name"], "daocloud");
}

#[test]
fn invalid_config_has_a_configuration_exit_code() {
    let path = std::env::temp_dir().join(format!(
        "lm-cli-invalid-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, "[unknown]\nvalue = true\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["--config", path.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn effective_config_is_machine_readable_and_redacts_credentials() {
    let path = std::env::temp_dir().join(format!(
        "lm-cli-config-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(
        &path,
        "[mirrors]\nprivate = \"https://example.com/simple?token=secret\"\n[defaults]\npip = \"private\"\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args([
            "--config",
            path.to_str().unwrap(),
            "config",
            "show",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["mirrors"]["private"], "https://example.com/simple");
    assert_eq!(value["defaults"]["pip"], "private");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("secret"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn target_mirror_pool_is_visible_in_effective_config() {
    let path = std::env::temp_dir().join(format!(
        "lm-cli-pool-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(
        &path,
        "[targets.pip]\ndefault = \"tuna\"\nmirrors = [\"tuna\", \"https://example.com/simple?token=secret\"]\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args([
            "--config",
            path.to_str().unwrap(),
            "config",
            "show",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["targets"]["pip"]["mirrors"][0], "tuna");
    assert_eq!(
        value["targets"]["pip"]["mirrors"][1],
        "https://example.com/simple"
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("secret"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn env_prints_shell_assignments_without_writing_a_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["env", "huggingface", "hf-mirror"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "export HF_ENDPOINT='https://hf-mirror.com'\n"
    );
}

#[test]
fn config_init_creates_a_template_without_overwriting() {
    let path = std::env::temp_dir().join(format!(
        "lm-cli-init-{}-{}.toml",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["--config", path.to_str().unwrap(), "config", "init"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(std::fs::read_to_string(&path)
        .unwrap()
        .contains("[options]"));

    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["--config", path.to_str().unwrap(), "config", "init"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn chsrc_target_aliases_and_plan_are_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["list", "lua", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["target"], "luarocks");

    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args([
            "plan",
            "lua",
            "https://mirror.example.com",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value[0]["schema"], "lm/v1");
    assert_eq!(value[0]["desired"], "https://mirror.example.com");
}

#[test]
fn invalid_parallelism_has_a_configuration_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["check", "docker", "--parallelism", "0"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn catalog_lint_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["--no-config", "catalog", "lint", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "lm/v1");
    assert_eq!(value["valid"], true);
    assert!(value["targets"].as_u64().unwrap() >= 70);
}

#[test]
fn config_sources_reports_disabled_loading() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["--no-config", "config", "sources", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "lm/v1");
    assert_eq!(value["sources"][0]["active"], false);
    assert_eq!(value["sources"][0]["loaded"], false);
}
