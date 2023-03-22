use std::process::Command;

pub fn set() {
    let output = Command::new("sh")
        .args([
            "-c",
            "gem sources --add https://mirrors.tuna.tsinghua.edu.cn/rubygems/ --remove https://rubygems.org/",
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn unset() {
    let output = Command::new("sh")
        .args(["-c", "gem sources --add https://rubygems.org/ --remove https://mirrors.tuna.tsinghua.edu.cn/rubygems/"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
