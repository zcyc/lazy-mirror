use std::{
    fs,
    io::{BufRead, BufReader, Write},
    path::Path,
};

pub fn maven_set() {
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

pub fn maven_unset() {
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

pub fn gradle_set() {
    // 获取 Gradle 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let gradle_config_path = home_dir.join(".gradle").join("init.gradle");

    // 如果配置文件不存在，则创建一个新的文件
    if !Path::new(&gradle_config_path).exists() {
        match fs::File::create(&gradle_config_path) {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Error: could not create Gradle configuration file: {}", e);
                return;
            }
        }
    }

    // 读取配置文件内容
    let file = fs::File::open(&gradle_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 repositories 部分，并添加新的 mirror
    let mut new_lines: Vec<String> = Vec::new();
    let mut in_repositories = false;
    let mut added_mirror = false;
    for line in lines {
        if line.contains("repositories {") {
            in_repositories = true;
            new_lines.push(line.clone());
            new_lines.push("    maven {".to_string());
            new_lines.push(
                "        url \"http://maven.aliyun.com/nexus/content/groups/public/\"".to_string(),
            );
            new_lines.push("    }".to_string());
        } else if in_repositories && !added_mirror && line.contains("mavenCentral()") {
            added_mirror = true;
            new_lines.push(line);
        } else if !added_mirror {
            new_lines.push(line);
        }
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&gradle_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully set the Gradle mirror.");
}

pub fn gradle_unset() {
    // 获取 Gradle 配置文件路径
    let home_dir = match dirs::home_dir() {
        Some(path) => path,
        None => {
            eprintln!("Error: could not find home directory.");
            return;
        }
    };
    let gradle_config_path = home_dir.join(".gradle").join("gradle.properties");

    // 如果配置文件不存在，则退出
    if !Path::new(&gradle_config_path).exists() {
        eprintln!("Error: could not find Gradle configuration file.");
        return;
    }

    // 读取配置文件内容
    let file = fs::File::open(&gradle_config_path).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().map(|l| l.unwrap()).collect();

    // 查找 mirrors 部分，并删除对应的 mirror
    let mut new_lines: Vec<String> = Vec::new();
    let mut skip_mirror = false;
    for line in lines {
        if line.clone().contains("systemProp.http.proxyHost")
            || line.clone().contains("systemProp.https.proxyHost")
        {
            skip_mirror = true;
        } else if !skip_mirror {
            new_lines.push(line.clone());
        }
        if line.clone().contains("systemProp.http.proxyPort")
            || line.clone().contains("systemProp.https.proxyPort")
        {
            skip_mirror = false;
        }
    }

    // 将新的配置写入文件
    let mut file = fs::File::create(&gradle_config_path).unwrap();
    for line in new_lines {
        writeln!(file, "{}", line).unwrap();
    }

    println!("Successfully unset the Gradle mirror.");
}
