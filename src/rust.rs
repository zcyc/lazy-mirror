use std::fs;
use std::io::Write;

pub fn set() {
    let config_content = r#"[source.crates-io]
# To use sparse index, change 'rsproxy' to 'rsproxy-sparse'
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
[source.rsproxy-sparse]
registry = "sparse+https://rsproxy.cn/index/"

[registries.rsproxy]
index = "https://rsproxy.cn/crates.io-index"

[net]
git-fetch-with-cli = true"#;

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cargo_config_path = format!("{}/.cargo/config", home_dir);

    // 删除已经存在的配置文件
    let _ = std::fs::remove_file(&cargo_config_path);

    // 创建新的配置文件并写入内容
    let mut cargo_config_file =
        fs::File::create(cargo_config_path).expect("Failed to create the Cargo config file.");
    cargo_config_file
        .write_all(config_content.as_bytes())
        .expect("Failed to write to the Cargo config file.");

    println!("Successfully set the Cargo mirror.");
}

pub fn unset() {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let cargo_config_path = format!("{}/.cargo/config", home_dir);

    _ = fs::remove_file(cargo_config_path);

    println!("Successfully unset the Cargo mirror.");
}
