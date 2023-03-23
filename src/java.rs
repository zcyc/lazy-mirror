use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::Path,
};

pub fn set() {
    // 获取 Maven 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let maven_config_path = home_dir.join(".m2").join("settings.xml");

    // 如果配置文件不存在，则退出
    if !Path::new(&maven_config_path).exists() {
        eprintln!("Error: could not find Maven configuration file.");
        return;
    }

    // 读取配置文件内容
    let file = fs::File::open(&maven_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 mirrors 部分，并替换它
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_mirror = false;
    let mut mirror_replaced = false;
    for line in lines {
        if line.contains("<mirrors>") {
            in_mirror = true;
        } else if in_mirror && line.contains("</mirrors>") {
            if !mirror_replaced {
                new_lines.push("        <mirror>".to_string());
                new_lines.push("            <id>aliyun</id>".to_string());
                new_lines.push("            <mirrorOf>*</mirrorOf>".to_string());
                new_lines.push(
                    "            <url>http://maven.aliyun.com/nexus/content/groups/public/</url>"
                        .to_string(),
                );
                new_lines.push("        </mirror>".to_string());
                mirror_replaced = true;
            }
            in_mirror = false;
        } else if !in_mirror {
            new_lines.push(line);
        }
    }

    // 如果没有找到 mirrors 部分，则添加它
    if !mirror_replaced {
        new_lines.push("    <mirrors>".to_string());
        new_lines.push("        <mirror>".to_string());
        new_lines.push("            <id>aliyun</id>".to_string());
        new_lines.push("            <mirrorOf>*</mirrorOf>".to_string());
        new_lines.push(
            "            <url>http://maven.aliyun.com/nexus/content/groups/public/</url>"
                .to_string(),
        );
        new_lines.push("        </mirror>".to_string());
        new_lines.push("    </mirrors>".to_string());
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&maven_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully set the Maven mirror.");
}

pub fn unset() {
    // 获取 Maven 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let maven_config_path = home_dir.join(".m2").join("settings.xml");

    // 如果配置文件不存在，则退出
    if !Path::new(&maven_config_path).exists() {
        eprintln!("Error: could not find Maven configuration file.");
        return;
    }

    // 读取配置文件内容
    let file = fs::File::open(&maven_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 mirrors 部分，并删除对应的 mirror
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_mirror = false;
    let mut skip_mirror = false;
    for line in lines {
        if line.contains("<mirrors>") {
            in_mirror = true;
        } else if line.contains("</mirrors>") {
            in_mirror = false;
            skip_mirror = false;
        } else if in_mirror && line.contains("<mirror>") {
            skip_mirror = false;
        } else if in_mirror && line.contains("</mirror>") && !skip_mirror {
            skip_mirror = true;
        } else if !skip_mirror {
            new_lines.push(line);
        }
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&maven_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully removed the Maven mirror.");
}
