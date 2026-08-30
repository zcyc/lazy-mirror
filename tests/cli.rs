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
fn targets_without_built_in_mirrors_are_not_listed() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["list", "helm", "--format", "json"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported target"));
}

#[test]
fn mutation_json_is_machine_readable_in_dry_run_mode() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["set", "docker", "daocloud", "--dry-run", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value[0]["schema"], "lm/v1");
    assert_eq!(value[0]["target"], "docker");
    assert_eq!(value[0]["changed"], true);
    assert_eq!(value[0]["dry_run"], true);
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
fn chsrc_target_aliases_and_plan_are_available() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["list", "node", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["target"], "npm");
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
fn invalid_selector_errors_do_not_leak_query_parameters() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args([
            "measure",
            "cargo",
            "sparse+https://mirror.example/index?token=secret",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let output = String::from_utf8_lossy(&output.stderr);
    assert!(output.contains("unknown built-in mirror"));
    assert!(!output.contains("token=secret"));
}

#[test]
fn config_file_and_direct_urls_are_not_supported() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["config", "init"])
        .output()
        .unwrap();
    assert!(!output.status.success());

    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["set", "pip", "https://mirror.example/simple", "--dry-run"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown built-in mirror"));
}

#[test]
fn catalog_lint_is_machine_readable() {
    let output = Command::new(env!("CARGO_BIN_EXE_lm"))
        .args(["catalog", "lint", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["schema"], "lm/v1");
    assert_eq!(value["valid"], true);
    assert_eq!(value["targets"], 42);
}
