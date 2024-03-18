<img width="256" alt="image" src="https://github.com/zcyc/lazy-mirror/assets/9925064/9ea87120-ecad-4b47-ae80-8dd80e02ea0a">

# lazy-mirror

一键设置开发环境镜像源

当前支持 Rust(Cargo)、Ruby(RubyGems/Bundler)、Python(pip)、PHP(Composer)、Node(NPM)、Java(Maven/Gradle)、Go

## 安装
```bash
cargo install --git https://github.com/zcyc/lazy-mirror
```

## 使用
```bash
lm set go
lm unset go
```

## 帮助
```
A mirror setting cli for lazy

Usage: lm <COMMAND>

Commands:
  set    
  unset  
  help   Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help information
```
