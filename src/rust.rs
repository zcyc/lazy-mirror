use std::{
    env,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

pub fn set() {
    // 镜像地址
    let mirror_url = &String::from("https://mirrors.tuna.tsinghua.edu.cn/git/cargo");

    // 获取 Cargo 配置文件路径
    let mut home_dir = match env::var_os("HOME") {
        Some(path) => PathBuf::from(path),
        None => PathBuf::from("."),
    };
    home_dir.push(".cargo");
    home_dir.push("config");

    // 写入镜像地址到 Cargo 配置文件
    let mut file = match File::open(&home_dir) {
        Ok(file) => file,
        Err(_) => {
            // 如果文件不存在，则创建一个新文件
            match File::create(&home_dir) {
                Ok(file) => file,
                Err(err) => {
                    println!("Failed to create file {}: {}", home_dir.display(), err);
                    return;
                }
            }
        }
    };

    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(err) => {
            println!("Failed to read file {}: {}", home_dir.display(), err);
            return;
        }
    }

    contents = contents.replace(
        "[source.crates-io]\nregistry = \"https://crates.io\"\n",
        &format!("[source.crates-io]\nregistry = \"{}\"\n", mirror_url),
    );
    match file.write_all(contents.as_bytes()) {
        Ok(_) => println!("Successfully set mirror to {}", mirror_url),
        Err(err) => println!("Failed to write to file {}: {}", home_dir.display(), err),
    }
}

pub fn unset() {
    let home_dir = env::var("HOME").expect("failed to get home directory");
    let cargo_config_path = format!("{}/.cargo/config", home_dir);

    let output = Command::new("sed")
        .arg("-i")
        .arg("/\\[registry\\.mirrors\\]/,/^\\[/ { /^[^#]/ s/^/#/ }")
        .arg(&cargo_config_path)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Successfully deleted Cargo registry mirror.");
    } else {
        println!("Failed to delete Cargo registry mirror.");
    }
}
