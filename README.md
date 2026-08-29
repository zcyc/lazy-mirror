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
lm get pip --explain           # 解释配置来源、mirror 池和 adapter 路径
lm get all --format json
lm get all --only-installed
lm get pip --all-scopes       # 查看各作用域的配置
lm doctor all --only-installed --explain  # 检查工具、配置和 mirror，并解释来源
lm plan docker daocloud         # 显示将要修改的路径和目标值
lm diff docker daocloud --format json
lm set pip --best --verify     # 选择最快可用源并在写入前复核
lm config validate
lm config show --format json
lm config sources               # 查看实际参与合并的配置文件
lm config sources --format json
lm config init                 # 创建最小 TOML 配置模板
lm --no-config list             # 忽略所有 TOML 配置
lm catalog lint --format json  # 校验内置目标、别名和 mirror
lm completions zsh > ~/.zfunc/_lm
lm set docker daocloud
lm set buildkit daocloud        # 写入 BuildKit registry mirror
lm set pip first              # 使用内置列表第一个源
lm set docker https://mirror.example.com
lm set cargo rsproxy --scope project --dry-run
lm set all --atomic             # 可回滚 adapter 的全量修改
lm reset docker
eval "$(lm env huggingface hf-mirror)"  # 输出当前 shell 可执行的环境变量
```

`list`、`measure`、`check`、`get` 支持 `--format table|json`；JSON 记录包含
`schema = "lm/v1"`。`get --all-scopes` 可一次查看各有效作用域的配置。
`measure`、`check` 和 `doctor` 支持 `--cache-ttl SECONDS`、`--no-cache`、
`--only-installed`、`--parallelism 1..64`、`--ipv4` 和 `--ipv6`；IP 选项互斥，默认自动选择。
`get`、`plan` 也支持 `--only-installed`。
`set` 支持 `--best` 自动选择最快可用源，以及 `--verify` 在写入前探测、写入后复核实际配置；复核失败会尝试恢复写入前状态。
成功时会显示 `verified`；失败信息会标记 `rollback=exact`、`rollback=attempted`，回滚本身失败时追加 `-failed`。
对 Node/Python/JVM/Rust/Dart/Haskell 等组合目标，复核会检查本次实际修改的每个已安装成员。
`set all`/`reset all` 默认跳过未安装或当前作用域不支持的目标；需要全量原子执行时使用
`set all --atomic`，它会对未安装目标直接失败。
命令别名为：`ls/l`、`m/cesu`、`verify`、`g`、`s`、`r`。

## TOML 配置

未指定 `--config` 且未设置 `LM_CONFIG` 时，按系统 → 用户 → 项目配置的顺序读取配置，后者
覆盖前者：系统为 `/etc/lazy-mirror/config.toml`（Windows 为
`C:\ProgramData\lazy-mirror\config.toml`），用户为 `$XDG_CONFIG_HOME/lazy-mirror/config.toml`
（macOS 通常是 `~/Library/Application Support/lazy-mirror/config.toml`），项目配置为当前目录
或最近父目录中的 `.lazy-mirror/config.toml`。`LM_CONFIG` 或 `--config FILE` 会改为只读取指定文件；
`--no-config` 完全跳过读取。`lm config sources` 显示路径、是否启用和是否成功加载。
`config show --format json` 与 `get --explain` 会标出配置项最终来源文件。

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

`targets.<name>.mirrors` 可定义该目标用于 `measure`、`--best` 的候选池；候选项可以是内置
镜像名、`[mirrors]` 名称或 URL。未配置候选池时使用该目标的全部内置镜像，并追加默认配置源。

```toml
[targets.pip]
default = "company"
mirrors = ["company", "tuna", "https://pypi.example.com/simple"]
```

## 检查与安全

`check` 按工具协议探测：Docker/BuildKit 使用 `/v2/`，Python 使用 `/simple/`，NuGet 使用
`/v3/index.json`，Hugging Face 使用 `/api/models?limit=1`。结果区分
`healthy`、`auth-required`、`rate-limited`、`unsupported`、`invalid-response`、`unavailable` 和网络错误；
一般只有 `healthy` 返回成功，Docker/containerd/Podman 的 `auth-required` 代表 Registry
已可达，也返回成功。JSON 结果还包含远端 IP、Content-Type、DNS、连接、TLS 和首字节耗时；
健康结果可以缓存到系统 cache 目录，或用 `LM_CACHE_FILE` 指定。

私有 mirror 的凭证只从环境变量读取，不写入配置或输出：

```bash
LM_MIRROR_TOKEN=... lm check pip https://packages.example.com --format json
LM_MIRROR_USERNAME=... LM_MIRROR_PASSWORD=... lm check docker https://registry.example.com
```

文件修改使用锁、临时文件和原子替换，并保留已有权限。原有用户配置会保存为
`.lazy-mirror.bak`；reset 发现受管文件被外部修改时会拒绝恢复。`set all` 会先预检所有
目标的选择器和作用域，再按顺序修改；执行期失败会立即停止，不会继续修改后续目标。
`set all --atomic` 只允许能读取并恢复旧 source 的 adapter，会在失败时按旧 source 逆序恢复；
无法证明可恢复的外部命令 adapter 会在预检阶段拒绝。

## 支持的项目

| 分类 | 项目 |
| --- | --- |
| Node.js | npm、pnpm、Yarn Classic、Yarn Berry、Bun |
| Python | pip、uv、PDM、Poetry |
| JVM | Maven、Gradle、sbt |
| 其他语言 | Go、Cargo/Rust、Rustup、RubyGems/Bundler、Composer/PHP、Conda/Mamba、CRAN/R、NuGet/.NET、Dart、Flutter、Hugging Face、Hex/Mix、Julia、CPAN/Perl、opam、Rye、nvm、LuaRocks、Clojure/Clojars、Haskell/Hackage/Cabal/Stack、OCaml |
| 系统/平台 | APT/Debian/Ubuntu、Alpine APK、Fedora、OpenSUSE、Kali、Arch、Manjaro、Gentoo、Rocky、Alma、Void、Solus、ROS、Raspberry Pi、Armbian、OpenWrt、openEuler、OpenAnolis、OpenKylin、Deepin、Linux Mint、MSYS2、Termux、FreeBSD、OpenBSD、NetBSD |
| 软件/桌面 | Homebrew、WinGet、CocoaPods、Flathub/Flatpak、Nix、Guix、Emacs/ELPA、TeX/CTAN |
| 容器 | Docker Engine、containerd/nerdctl、Podman |

Dart、Flutter、Hugging Face、Homebrew、Rustup、Julia、CPAN、Rye、nvm、Nix、Guix 使用受管
shell 环境变量块；`lm env` 可只输出变量而不修改文件，支持 sh、fish、PowerShell。
项目作用域写入 `.env`，用户作用域默认写入 `.profile`，也可用 `LM_SHELL_PROFILE` 指定。
Cargo 和 uv 使用 TOML 结构化合并，项目 scope 会分别识别最近父目录的 `.cargo/config.toml`/`.cargo/config`
和 `uv.toml`/`pyproject.toml`（同目录的 `uv.toml` 优先）；Docker 和 BuildKit 使用 JSON/TOML 结构化合并。

`lm catalog lint` 不联网，只校验内置目标、全局选择器唯一性、mirror 名称和 URL；建议在 CI
中运行，避免新增目标或 mirror 时破坏命令解析。

APT 默认写入 `/etc/apt/sources.list.d/lazy-mirror.list`，可用 `LM_APT_SOURCES_FILE`
覆盖；发行版会从 `/etc/os-release` 的 `VERSION_CODENAME`/`UBUNTU_CODENAME` 推断，
也可用 `LM_APT_DISTRIBUTION`、`LM_APT_SUITES`、`LM_APT_COMPONENTS` 覆盖。Alpine 使用
`LM_APK_REPOSITORIES_FILE` 或 `/etc/apk/repositories`。系统源只支持 `--scope system`，
不会自动执行 apt/apk 更新。

没有稳定内置源语义的项目仍然支持 URL/TOML 覆盖，但不会伪造内置 mirror。系统发行版目标
同样只接受显式 URL 或 `[mirrors]` 配置，并且只在指定 scope 写入。

Gentoo 使用受管的 `GENTOO_MIRRORS` 配置块，ROS 使用带发行版代号的 APT 源；可用
`LM_ROS_DISTRIBUTION` 覆盖代号。系统平台配置仍要求显式 `--scope system`。

MSYS2 根据 `MSYSTEM` 选择对应的 `mirrorlist.*`；可用 `LM_MSYS2_MIRRORLIST` 指定文件。
Termux 要求 `PREFIX` 环境变量，避免在错误目录创建仓库配置。

## Docker 与 Hugging Face

Docker 默认镜像来自 [DaoCloud public-image-mirror](https://github.com/DaoCloud/public-image-mirror)，
内置源仅保留 `daocloud`；仍可传任意 HTTP(S) URL 覆盖内置选择。
Docker、BuildKit（`buildctl`/`docker buildx`）、containerd/nerdctl、Podman 会按
`user`/`system` scope 分别写入用户或系统配置。

```bash
lm list docker
lm set docker daocloud
lm set docker https://docker.example.com
lm reset docker

lm list buildkit
lm set buildkit daocloud
lm reset buildkit
```

Docker 镜像源必须是 registry 根 URL，例如 `https://docker.example.com`；带路径、查询参数
的值会被拒绝。Docker Engine 使用 `registry-mirrors`，遵循
[Docker daemon mirror 配置](https://docs.docker.com/docker-hub/image-library/mirror/)；
BuildKit 使用以下结构，遵循
[BuildKit registry mirror 配置](https://docs.docker.com/build/buildkit/configure/)：

```toml
[registry."docker.io"]
mirrors = ["https://docker.m.daocloud.io"]
```

BuildKit 默认写入 `~/.config/buildkit/buildkitd.toml` 或 `/etc/buildkit/buildkitd.toml`，
可用 `LM_BUILDKIT_CONFIG` 覆盖。`lm get buildkit` 会检查 `buildctl` 或 `docker buildx`；
使用 Buildx 时，把该文件传给 builder：

```bash
docker buildx create --use --bootstrap --name lm-builder \
  --driver docker-container --buildkitd-config ~/.config/buildkit/buildkitd.toml
```

BuildKit 是独立目标，不包含在 `set all` 中；需要显式执行 `lm set buildkit ...`，避免意外
改动 daemon 和 BuildKit 两套运行时配置。

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

命令形状已对齐 [chsrc](https://github.com/RubyMetric/chsrc) 的
`list/measure/get/set/reset`、镜像名/URL 覆盖、`first`、dry-run、作用域
和 JSON 自动化输出，并补充 `doctor`、`plan/diff`、`config validate/show`。目标和别名
覆盖 chsrc 当前公开清单；没有稳定官方 mirror 语义的工具允许自定义 URL，但不会伪造
内置源。

这是一次破坏性 CLI 重构：旧的 `unset`、`status` 命令，以及
`lm set docker --mirror NAME` 形式已删除；旧配置不会自动迁移。Docker/BuildKit 现在拒绝
带路径或查询参数的 mirror URL；健康缓存键新增 IP 版本和探测指标，旧缓存会直接失效，
不做迁移。发布物同时生成 SHA256、SPDX SBOM 和 GitHub provenance attestation，可用
[`gh attestation verify`](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/manage-attestations)
校验。`doctor` 是新的诊断命令。
