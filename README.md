# lazy-mirror

切换包管理器和开发工具镜像源的 CLI，命令名为 `lm`。

## 安装

```bash
cargo install --git https://github.com/zcyc/lazy-mirror
```

## 快速上手

```bash
# 查看支持的工具和镜像
lm list
lm list pip

# 测速或检查镜像
lm measure pip
lm check pip

# 查看当前配置
lm get pip

# 切换镜像：可以使用内置名称或 HTTP(S) URL
lm set pip tuna
lm set docker https://docker.example.com

# 自动选择最快镜像，并在写入前后校验
lm set pip --best --verify

# 预览修改、恢复默认源
lm plan pip tuna
lm reset pip
```

`measure` 用于比较速度，`check` 用于按工具协议检查镜像是否可用。

## 命令速查

| 命令 | 用途 |
| --- | --- |
| `lm list [target]` | 列出工具或指定工具的镜像 |
| `lm measure <target> [mirror]` | 测量镜像可用性和延迟 |
| `lm check <target> [mirror]` | 检查镜像协议端点 |
| `lm get <target>` | 查看当前配置；`--explain` 可显示来源 |
| `lm set <target> [mirror]` | 写入镜像名称或 URL |
| `lm reset <target>` | 恢复上游默认源 |
| `lm plan <target> [mirror]` | 查看将要修改的路径和值 |
| `lm doctor <target>` | 检查工具、配置和镜像 |
| `lm config init` | 创建配置模板 |
| `lm config validate` | 校验配置 |
| `lm config show` | 查看生效配置 |
| `lm config sources` | 查看配置文件来源 |
| `lm catalog lint` | 校验内置目标和镜像目录 |
| `lm env <target> [mirror]` | 输出当前 shell 可执行的环境变量 |
| `lm completions <shell>` | 生成 Bash、Zsh、Fish 或 PowerShell 补全 |

常用选项：

```text
--scope project|user|system   写入项目、用户或系统范围（默认 user）
--dry-run                     只预览，不修改文件
--format table|json           选择输出格式
--only-installed              只处理已安装的工具
--verify                      写入前探测，写入后复核
--best                        自动选择最快可用镜像
--atomic                      set all 时失败则回滚
```

目标和镜像名称以 `lm list` 的输出为准。没有内置镜像的目标可以直接传 URL。

## 配置

```bash
lm config init
lm config validate
lm config show
```

配置文件使用 TOML。未指定路径时按系统、用户、项目的顺序读取，后者覆盖前者；也可以用
`--config FILE` 指定单个文件，或用 `--no-config` 完全忽略配置。

```toml
[mirrors]
company = "https://packages.example.com/simple"

[defaults]
pip = "company"
docker = "https://registry.example.com"

[options]
timeout_seconds = 10
retries = 1
parallelism = 4
```

## 常用场景

### Docker / BuildKit

```bash
lm list docker
lm set docker daocloud
lm set buildkit daocloud
lm reset docker
lm reset buildkit
```

### Hugging Face

```bash
lm set huggingface hf-mirror
eval "$(lm env huggingface hf-mirror)"
```

### JSON 输出

```bash
lm list pip --format json
lm get all --format json
lm set pip tuna --format json
```

新版命令已对齐 [chsrc](https://github.com/RubyMetric/chsrc) 。
