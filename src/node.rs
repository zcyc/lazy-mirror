pub fn set(name: &String) {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            format!(
                "{} {}",
                name,
                String::from("config set registry https://registry.npmmirror.com/")
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
            format!(
                "{} {}",
                name,
                String::from("config set registry https://registry.npmjs.org/")
            )
            .as_ref(),
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
