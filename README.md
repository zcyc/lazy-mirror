# lazy-mirror

面向开发工具链的镜像源切换 CLI，命令风格对齐 [chsrc](https://github.com/RubyMetric/chsrc)。

## 安装

~~~bash
cargo install --git https://github.com/zcyc/lazy-mirror
~~~

## 命令

~~~bash
lm list                         # 列出支持的项目
lm list docker                  # 列出项目的内置镜像
lm list mirror                  # 列出所有镜像名称
lm measure docker               # curl 检测所有 Docker 镜像并显示 HTTP 状态/耗时
lm measure docker daocloud      # 只检测一个镜像
lm get go                       # 查看当前配置
lm set docker daocloud
lm set docker https://mirror.example.com
lm set cargo rsproxy --scope project --dry-run
lm reset docker
~~~

list、measure、get、set 分别支持 ls/l、m/cesu、g、s 别名。--config FILE 可指定 TOML 配置文件。

### TOML 配置

默认位置是 $XDG_CONFIG_HOME/lazy-mirror/config.toml，macOS 通常是
~/Library/Application Support/lazy-mirror/config.toml；也可以使用 LM_CONFIG 或
--config FILE 覆盖。

~~~toml
[mirrors]
company = "https://packages.example.com/simple"
docker-internal = { url = "https://docker.example.com" }

[defaults]
pip = "company"
docker = "docker-internal"
~~~

[mirrors] 用名称定义可复用 URL，[defaults] 为项目选择默认镜像；默认值也可以直接写
https://...。命令行最后的镜像名称或 URL 优先级最高，因此可以临时覆盖 TOML 默认值。

### 作用域和安全行为

~~~bash
lm set npm npmmirror --scope user
lm set cargo rsproxy --scope project
lm set poetry tuna --scope project
lm reset cargo --scope project
~~~

支持 project、user、system。作用域是否由具体工具支持取决于其官方配置机制；
不支持时会明确报错。Cargo 和 uv 使用 TOML 结构化合并，Docker 使用 JSON 合并；其余需要
替换独立配置文件的工具会在同目录保存 .lazy-mirror.bak。reset 只恢复仍由
lazy-mirror 管理的内容，发现受管文件被外部修改时会拒绝操作。

measure 需要系统安装 curl，将跟随重定向并报告 HTTP 状态和耗时；404、5xx、
超时或无法建立连接会使命令返回非零退出码。它只检查可达性，不代表镜像内容完整或长期可用。

## 支持的项目

| 分类 | 项目 |
| --- | --- |
| Node.js | npm、pnpm、Yarn Classic、Yarn Berry、Bun |
| Python | pip、uv、PDM、Poetry |
| JVM | Maven、Gradle、sbt |
| 其他语言 | Go、Cargo/Rust、RubyGems/Bundler、Composer/PHP、Conda/Mamba、CRAN/R、NuGet/.NET、Dart、Flutter、Hugging Face |
| 容器 | Docker Engine、containerd/nerdctl、Podman |

Dart 和 Flutter 按官方机制写入 PUB_HOSTED_URL、FLUTTER_STORAGE_BASE_URL 的受管环境变量
块：用户级默认写入当前 shell 的 profile，项目级写入 .env，系统级写入 /etc/profile。
Hugging Face 写入 HF_ENDPOINT，支持 huggingface、hf、huggingface-hub 别名。

## Docker

Docker 默认镜像来自 [DaoCloud public-image-mirror](https://github.com/DaoCloud/public-image-mirror)，
内置源仅保留 daocloud；仍可传任意 HTTP(S) URL 覆盖内置选择。

~~~bash
lm list docker
lm set docker daocloud
lm set docker https://docker.example.com
lm reset docker
~~~

Hugging Face 使用 HF_ENDPOINT 指向 Hub 地址；issue #219 推荐的内置源是
[hf-mirror.com](https://hf-mirror.com/)：

~~~bash
lm list huggingface
lm set huggingface hf-mirror
lm set hf https://hf.example.com --scope project
~~~

默认配置路径遵循 Docker 官方约定：

- Linux：/etc/docker/daemon.json
- Linux rootless：$XDG_CONFIG_HOME/docker/daemon.json 或 ~/.config/docker/daemon.json
- Docker Desktop/macOS：~/.docker/daemon.json
- Windows：C:\ProgramData\docker\config\daemon.json

可用 LM_DOCKER_DAEMON_CONFIG 覆盖路径。工具只写 registry-mirrors，保留同一 JSON
文件中的其他键，不会自动重启 Docker；修改 daemon 配置后请自行重启 Docker。Docker
官方和 DaoCloud 均提醒：registry-mirrors 适用于 Docker Hub，不应把普通软件包镜像填入其中。

containerd 使用 /etc/containerd/certs.d/docker.io/hosts.toml，可用
LM_CONTAINERD_HOSTS_FILE 覆盖；Podman 使用 /etc/containers/registries.conf，可用
LM_PODMAN_CONFIG 覆盖。

## 与 chsrc 的边界

命令形状已对齐 list/measure/get/set/reset、镜像名/URL 覆盖、dry-run 和作用域。
当前优先覆盖常见开发工具与容器运行时；APT、Homebrew、Winget、CPAN、Lua、Julia、
Haskell、OCaml、Linux 发行版等需要各自的系统文件或包管理语义，尚未加入可写适配器，
避免把“能访问镜像”误当成“能安全修改系统源”。

这是一次破坏性 CLI 重构：旧的 unset、status、doctor 命令，以及
lm set docker --mirror NAME 形式已删除；请改用 reset、get 和 lm set docker NAME。
旧配置不会自动迁移。
