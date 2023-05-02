use std::fs;
use std::io::Write;

pub fn maven_set() {
    let config_content = r#"<?xml version="1.0" encoding="utf-8"?>
<settings xmlns="http://maven.apache.org/SETTINGS/1.0.0"
    xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
    xsi:schemaLocation="http://maven.apache.org/SETTINGS/1.0.0 http://maven.apache.org/xsd/settings-1.0.0.xsd">
    <mirrors>
        <mirror>
            <id>aliyunmaven</id>
            <mirrorOf>*</mirrorOf>
            <name>阿里云公共仓库</name>
            <url>https://maven.aliyun.com/repository/public</url>
        </mirror>
    </mirrors>
</settings>"#;

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let maven_config_path = format!("{}/.m2/settings.xml", home_dir);

    // 删除已经存在的配置文件
    let _ = std::fs::remove_file(&maven_config_path);

    // 创建新的配置文件并写入内容
    let mut maven_config_file = fs::File::create(&maven_config_path).unwrap();
    maven_config_file
        .write_all(config_content.as_bytes())
        .unwrap();

    println!("Successfully set the Maven mirror.");
}

pub fn maven_unset() {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let maven_config_path = format!("{}/.m2/settings.xml", home_dir);

    // 删除已经存在的配置文件
    let _ = std::fs::remove_file(maven_config_path);

    println!("Successfully deleted the Maven configuration file.");
}

pub fn gradle_set() {
    let gradle_content = r#"allprojects {
  repositories {
    maven {
      url 'https://maven.aliyun.com/repository/public/'
    }
    mavenLocal()
    mavenCentral()
  }
}"#;

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let gradle_config_path = format!("{}/.gradle/init.gradle", home_dir);

    // 删除已经存在的配置文件
    let _ = std::fs::remove_file(&gradle_config_path);

    // 创建并写入 Gradle 配置文件
    let mut gradle_config_file = fs::File::create(gradle_config_path).unwrap();
    gradle_config_file
        .write_all(gradle_content.as_bytes())
        .unwrap();

    println!("Successfully set the Gradle mirror.");
}

pub fn gradle_unset() {
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let gradle_config_path = format!("{}/.gradle/init.gradle", home_dir);

    if std::path::Path::new(&gradle_config_path).exists() {
        std::fs::remove_file(&gradle_config_path).unwrap();
        println!("Successfully unset the Gradle mirror.");
    } else {
        println!("Gradle mirror is not set.");
    }
}
