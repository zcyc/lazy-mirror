pub fn set() {
    let zshrc = r#"# set by lirror begin #
    export RUSTUP_DIST_SERVER="https://rsproxy.cn"
    export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
    # set by lirror end #"#;
    // let new_zshrc = zshrc.replace(zshrc, "new string");
    // println!("new_zshrc: {new_zshrc}");
    let cmd = format!("echo '{zshrc}' >> ~/kk");
    println!("cmd: {cmd}");
    let output = std::process::Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .expect("set cargo error");
    println!("output: {output:?}");
}

pub fn unset() {
    todo!()
}
