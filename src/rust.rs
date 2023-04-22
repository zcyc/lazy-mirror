use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::Path,
};

pub fn set() {
    // 获取 Cargo 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let cargo_config_path = home_dir.join(".cargo").join("config");

    // 如果配置文件不存在，则创建一个空的配置文件
    if !Path::new(&cargo_config_path).exists() {
        fs::create_dir_all(cargo_config_path.parent().unwrap()).unwrap();
        fs::File::create(&cargo_config_path).unwrap();
    }

    // 读取配置文件内容
    let file = fs::File::open(&cargo_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 registry 的位置，并替换为镜像
    let mut new_lines: Vec<String> = Vec::new();
    let mut found_registry = false;
    for line in lines {
        if line.starts_with("[source.crates-io]") {
            new_lines.push(line.to_owned());
            new_lines.push("replace-with = 'rsproxy'".to_owned());
            found_registry = true;
        } else {
            new_lines.push(line);
        }
    }
    if !found_registry {
        new_lines.push("[source.crates-io]".to_owned());
        new_lines.push("replace-with = 'rsproxy'".to_owned());
    }

    // 添加镜像配置
    new_lines.push("[source.rsproxy]".to_owned());
    new_lines.push("registry = 'https://rsproxy.cn/crates.io-index'".to_owned());

    new_lines.push("[source.rsproxy-sparse]".to_owned());
    new_lines.push("registry = 'sparse+https://rsproxy.cn/index/'".to_owned());

    new_lines.push("[registries.rsproxy]".to_owned());
    new_lines.push("index = 'https://rsproxy.cn/crates.io-index'".to_owned());

    new_lines.push("[net]".to_owned());
    new_lines.push("git-fetch-with-cli = true".to_owned());

    // 将新的配置写入文件
    let mut file = fs::File::create(&cargo_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully set the Cargo mirror.");
}

pub fn unset() {
    // 获取 Cargo 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let cargo_config_path = home_dir.join(".cargo").join("config");

    // 如果配置文件不存在，则直接返回
    if !Path::new(&cargo_config_path).exists() {
        eprintln!("Error: .cargo/config file does not exist.");
        return;
    }

    // 读取配置文件内容
    let file = fs::File::open(&cargo_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 搜索要删除的内容，并将其从配置文件中移除
    let mut deleted_lines = 0;
    let deleted_content = vec![
        "[source.crates-io]".to_string(),
        "replace-with = 'rsproxy'".to_string(),
        "".to_string(),
        "[source.rsproxy]".to_string(),
        "registry = \"https://rsproxy.cn/crates.io-index\"".to_string(),
        "".to_string(),
        "[source.rsproxy-sparse]".to_string(),
        "registry = \"sparse+https://rsproxy.cn/index/\"".to_string(),
        "".to_string(),
        "[registries.rsproxy]".to_string(),
        "index = \"https://rsproxy.cn/crates.io-index\"".to_string(),
        "".to_string(),
        "[net]".to_string(),
        "git-fetch-with-cli = true".to_string(),
    ];
    let mut new_lines: Vec<String> = Vec::new();
    for line in lines {
        if deleted_content.contains(&line) {
            deleted_lines += 1;
        } else {
            new_lines.push(line);
        }
    }

    // 如果没有删除任何内容，则直接返回
    if deleted_lines == 0 {
        eprintln!("Error: specified content does not exist in .cargo/config file.");
        return;
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&cargo_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully unset the Cargo mirror.");
}
