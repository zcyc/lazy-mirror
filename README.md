# lazy-mirror

面向开发工具链的镜像源切换 CLI，命令风格对齐 [chsrc](https://github.com/RubyMetric/chsrc)。

## 安装

```bash
cargo install --git https://github.com/zcyc/lazy-mirror
```

## 命令

```bash
lm list                         # 列出支持的项目
lm list docker                  # 列出项目的内置镜像
lm list mirror --format json   # 列出镜像和 URL
lm measure docker               # 并发检测镜像并显示 HTTP 状态/耗时
lm check docker --format json  # 按 Docker Registry API 路径校验
lm get go                      # 查看当前配置、实际 source 和作用域
lm get all --format json
lm set docker daocloud
lm set pip first              # 使用内置列表第一个源
lm set docker https://mirror.example.com
lm set cargo rsproxy --scope project --dry-run
lm reset docker
```

`list`、`measure`、`check`、`get` 支持 `--format table|json`。
`measure` 和 `check` 支持 `--cache-ttl SECONDS`、`--no-cache`。
命令别名为：`ls/l`、`m/cesu`、`verify`、`g`、`s`、`r`。

## TOML 配置

默认位置是 `$XDG_CONFIG_HOME/lazy-mirror/config.toml`，macOS 通常是
`~/Library/Application Support/lazy-mirror/config.toml`；也可以使用 `LM_CONFIG` 或
`--config FILE` 覆盖。

```toml
[mirrors]
company = "https://packages.example.com/simple"
docker-internal = { url = "https://docker.example.com" }

[defaults]
pip = "company"
docker = "docker-internal"

[targets.uv]
default = "company"
enabled = true

[options]
timeout_seconds = 10
retries = 1
cache_ttl_seconds = 300
parallelism = 4
```

`[mirrors]` 定义可复用 URL；`[defaults]` 或 `[targets.<name>]` 选择默认镜像；命令行
最后的镜像名称或 URL 优先级最高。镜像 URL 必须是 HTTP(S)。`targets.<name>.enabled=false`
可让 `all` 跳过目标。配置只接受 `mirrors`、`defaults`、`targets`、`options` 四个顶层
区块，错误会直接返回，不会静默降级。

## 检查与安全

`check` 按工具协议探测：Docker 使用 `/v2/`，Python 使用 `/simple/`，NuGet 使用
`/v3/index.json`，Hugging Face 使用 `/api/models?limit=1`。结果区分
`healthy`、`auth-required`、`rate-limited`、`unsupported`、`unavailable` 和网络错误；
只有 `healthy` 返回成功。健康结果可以缓存到系统 cache 目录，或用 `LM_CACHE_FILE` 指定。

私有 mirror 的凭证只从环境变量读取，不写入配置或输出：

```bash
LM_MIRROR_TOKEN=... lm check pip https://packages.example.com --format json
LM_MIRROR_USERNAME=... LM_MIRROR_PASSWORD=... lm check docker https://registry.example.com
```

文件修改使用锁、临时文件和原子替换，并保留已有权限。原有用户配置会保存为
`.lazy-mirror.bak`；reset 发现受管文件被外部修改时会拒绝恢复。`set all` 会先预检所有
目标的选择器和作用域，再按顺序修改；执行期失败会立即停止，不会继续修改后续目标。

## 支持的项目

| 分类 | 项目 |
| --- | --- |
| Node.js | npm、pnpm、Yarn Classic、Yarn Berry、Bun |
| Python | pip、uv、PDM、Poetry |
| JVM | Maven、Gradle、sbt |
| 其他语言 | Go、Cargo/Rust、Rustup、RubyGems/Bundler、Composer/PHP、Conda/Mamba、CRAN/R、NuGet/.NET、Dart、Flutter、Hugging Face、Hex/Mix、Julia、CPAN/Perl、opam |
| 系统/平台 | APT/Debian/Ubuntu、Alpine APK、Homebrew、WinGet |
| 容器 | Docker Engine、containerd/nerdctl、Podman |

Dart、Flutter、Hugging Face、Homebrew、Rustup、Julia、CPAN 使用受管 shell 环境变量块；
项目作用域写入 `.env`，用户作用域默认写入 `.profile`，也可用 `LM_SHELL_PROFILE` 指定。
Cargo 和 uv 使用 TOML 结构化合并，Docker 使用 JSON 合并。

APT 默认写入 `/etc/apt/sources.list.d/lazy-mirror.list`，可用 `LM_APT_SOURCES_FILE`
覆盖；发行版和组件可用 `LM_APT_DISTRIBUTION`、`LM_APT_COMPONENTS` 覆盖。Alpine 使用
`LM_APK_REPOSITORIES_FILE` 或 `/etc/apk/repositories`。系统源只支持 `--scope system`，
不会自动执行 apt/apk 更新。

## Docker 与 Hugging Face

Docker 默认镜像来自 [DaoCloud public-image-mirror](https://github.com/DaoCloud/public-image-mirror)，
内置源仅保留 `daocloud`；仍可传任意 HTTP(S) URL 覆盖内置选择。

```bash
lm list docker
lm set docker daocloud
lm set docker https://docker.example.com
lm reset docker
```

默认配置路径遵循 Docker 官方约定：Linux `/etc/docker/daemon.json`，Linux rootless 使用
`$XDG_CONFIG_HOME/docker/daemon.json` 或 `~/.config/docker/daemon.json`，Docker Desktop
使用 `~/.docker/daemon.json`，Windows 使用 `C:\ProgramData\docker\config\daemon.json`。
可用 `LM_DOCKER_DAEMON_CONFIG` 覆盖。工具只写 `registry-mirrors`，保留同一 JSON 中的
其他键，不会自动重启 Docker。

Hugging Face 使用 `HF_ENDPOINT`；issue #219 推荐的内置源是
[hf-mirror.com](https://hf-mirror.com/)：

```bash
lm list huggingface
lm set huggingface hf-mirror
lm set hf https://hf.example.com --scope project
```

## 与 chsrc 的边界

命令形状已对齐 `list/measure/get/set/reset`、镜像名/URL 覆盖、dry-run、作用域和 JSON
自动化输出。当前内置覆盖常见开发工具、容器运行时和主要系统平台；没有稳定官方 mirror
语义的工具允许自定义 URL，但不会伪造内置源。

这是一次破坏性 CLI 重构：旧的 `unset`、`status`、`doctor` 命令，以及
`lm set docker --mirror NAME` 形式已删除；旧配置不会自动迁移。
