pub fn set() {
    let output = std::process::Command::new("sh")
        .args(["-c", "go env -w GOPROXY=https://goproxy.cn,direct"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn unset() {
    let output = std::process::Command::new("sh")
        .args(["-c", "go env -u GOPROXY"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
