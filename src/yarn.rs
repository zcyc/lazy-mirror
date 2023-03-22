pub fn set() {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            "yarn config set registry https://registry.npmmirror.com/",
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn unset() {
    let output = std::process::Command::new("sh")
        .args(["-c", "yarn config set registry https://registry.npmjs.org/"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
