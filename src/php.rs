pub fn set() {
    let output = std::process::Command::new("sh")
        .args([
            "-c",
            "composer config -g repo.packagist composer https://mirrors.aliyun.com/composer/",
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn unset() {
    let output = std::process::Command::new("sh")
        .args(["-c", "composer config -g --unset repos.packagist"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
