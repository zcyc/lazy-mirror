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
        if line.starts_with(r#"[registry."#) {
            new_lines.push("[registry.crates-io]".to_string());
            new_lines.push(
                "index = \"https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git\""
                    .to_string(),
            );
            found_registry = true;
        } else {
            new_lines.push(line);
        }
    }
    if !found_registry {
        new_lines.push("[registry.crates-io]".to_string());
        new_lines.push(
            "index = \"https://mirrors.tuna.tsinghua.edu.cn/git/crates.io-index.git\"".to_string(),
        );
    }

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

    // 如果配置文件不存在，则退出
    if !Path::new(&cargo_config_path).exists() {
        eprintln!("Error: could not find Cargo configuration file.");
        return;
    }

    // 读取配置文件内容
    let file = fs::File::open(&cargo_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 registry 部分，并删除它
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_registry = false;
    for line in lines {
        if line.starts_with(r#"[registry."#) {
            in_registry = true;
        } else if in_registry && line.starts_with('[') {
            in_registry = false;
        } else if !in_registry {
            new_lines.push(line);
        }
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&cargo_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully removed the Cargo mirror.");
}
