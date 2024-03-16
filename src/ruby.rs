use std::process::Command;

pub fn gem_set() {
    let output = Command::new("sh")
        .args([
            "-c",
            "gem sources --add https://gems.ruby-china.com/ --remove https://rubygems.org/",
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn gem_unset() {
    let output = Command::new("sh")
        .args(["-c", "gem sources --add https://rubygems.org/ --remove https://gems.ruby-china.com/"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn bundle_set() {
    let output = Command::new("sh")
        .args([
            "-c",
            "bundle config mirror.https://rubygems.org https://gems.ruby-china.com",
        ])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}

pub fn bundle_unset() {
    let output = Command::new("sh")
        .args(["-c", "bundle config  --delete 'mirror.https://rubygems.org/'"])
        .output()
        .expect("failed to execute");
    println!("{:?}", output);
}
