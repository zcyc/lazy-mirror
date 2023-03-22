pub fn set(name: &String) {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            format!(
                "{} {}",
                name,
                String::from(
                    "config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple"
                )
            )
            .as_ref(),
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn unset(name: &String) {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            format!("{} {}", name, String::from("config unset global.index-url")).as_ref(),
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
