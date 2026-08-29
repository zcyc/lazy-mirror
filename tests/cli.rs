use std::process::Command;

#[test]
fn list_json_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["list", "docker", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
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
