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

# 切换到内置镜像
lm set pip tuna
lm set docker daocloud

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
| `lm set <target> [mirror]` | 写入内置镜像 |
| `lm reset <target>` | 恢复上游默认源 |
| `lm plan <target> [mirror]` | 查看将要修改的路径和值 |
| `lm doctor <target>` | 检查工具、配置和镜像 |
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

目标和镜像名称以 `lm list` 的输出为准。源目录固定在程序内置目录中，不读取配置文件，也不接受自定义 URL。

## 支持的目标和源

当前内置目录包含 **77 个目标**、**161 个源条目**。源数量为 0 的目标已列出，但当前没有可供 `lm set` 选择的内置源。

### 目标

| 目标 | 别名 | 内置源数量 |
| --- | --- | ---: |
| `npm` | `node`, `nodejs` | 6 |
| `pnpm` | — | 6 |
| `yarn` | — | 6 |
| `bun` | — | 6 |
| `go` | — | 5 |
| `pip` | `pip3`, `python`, `py`, `pypi` | 10 |
| `uv` | — | 10 |
| `pdm` | — | 10 |
| `poetry` | — | 10 |
| `composer` | `php` | 2 |
| `gem` | `ruby` | 6 |
| `bundle` | `bundler` | 6 |
| `maven` | `java`, `mvn`, `maven-daemon`, `mvnd` | 5 |
| `gradle` | — | 5 |
| `sbt` | — | 5 |
| `cargo` | `rust`, `crate` | 4 |
| `docker` | `dockerhub` | 1 |
| `buildkit` | `docker-buildkit`, `buildx` | 1 |
| `containerd` | `nerdctl` | 1 |
| `podman` | — | 1 |
| `helm` | — | 0 |
| `conda` | `mamba`, `anaconda` | 4 |
| `nuget` | `dotnet` | 1 |
| `dart` | `pub` | 3 |
| `flutter` | — | 2 |
| `cran` | `r` | 2 |
| `huggingface` | `hf`, `huggingface-hub` | 1 |
| `apt` | `debian`, `ubuntu` | 5 |
| `apk` | `alpine` | 5 |
| `brew` | `homebrew` | 1 |
| `rustup` | — | 5 |
| `hex` | `mix` | 0 |
| `julia` | — | 0 |
| `cpan` | `perl` | 2 |
| `winget` | — | 1 |
| `opam` | — | 0 |
| `rye` | — | 0 |
| `nvm` | — | 5 |
| `luarocks` | `lua` | 0 |
| `clojure` | `clojars` | 1 |
| `haskell` | `hackage` | 2 |
| `cabal` | — | 2 |
| `stack` | — | 2 |
| `ocaml` | — | 0 |
| `cocoapods` | `cocoa`, `pod` | 1 |
| `flathub` | `flatpak` | 1 |
| `nix` | — | 3 |
| `guix` | — | 0 |
| `emacs` | `elpa` | 3 |
| `tex` | `ctan`, `latex`, `texlive`, `miktex` | 3 |
| `linuxmint` | `mint`, `zorinos` | 0 |
| `fedora` | — | 0 |
| `opensuse` | `suse` | 0 |
| `kali` | — | 0 |
| `arch` | `archlinux` | 0 |
| `archlinuxcn` | — | 0 |
| `manjaro` | — | 0 |
| `gentoo` | — | 0 |
| `rocky` | `rockylinux` | 0 |
| `alma` | `almalinux` | 0 |
| `voidlinux` | `void` | 0 |
| `solus` | — | 0 |
| `ros` | `ros2` | 0 |
| `trisquel` | — | 0 |
| `linuxlite` | `lite` | 0 |
| `raspi` | `raspberrypi` | 0 |
| `armbian` | — | 0 |
| `openwrt` | — | 0 |
| `openeuler` | — | 0 |
| `openanolis` | `anolis` | 0 |
| `openkylin` | — | 0 |
| `deepin` | — | 0 |
| `msys2` | `msys` | 0 |
| `termux` | — | 0 |
| `freebsd` | — | 0 |
| `openbsd` | — | 0 |
| `netbsd` | — | 0 |

### 内置源明细

| 目标 | 源名 | 地址 |
| --- | --- | --- |
| `npm` | `npm` | https://registry.npmjs.org/ |
| `npm` | `yarn` | https://registry.yarnpkg.com/ |
| `npm` | `npmmirror` | https://registry.npmmirror.com/ |
| `npm` | `taobao` | https://registry.npmmirror.com/ |
| `npm` | `tencent` | https://mirrors.tencent.com/npm/ |
| `npm` | `huawei` | https://repo.huaweicloud.com/repository/npm/ |
| `pnpm` | `npm` | https://registry.npmjs.org/ |
| `pnpm` | `yarn` | https://registry.yarnpkg.com/ |
| `pnpm` | `npmmirror` | https://registry.npmmirror.com/ |
| `pnpm` | `taobao` | https://registry.npmmirror.com/ |
| `pnpm` | `tencent` | https://mirrors.tencent.com/npm/ |
| `pnpm` | `huawei` | https://repo.huaweicloud.com/repository/npm/ |
| `yarn` | `npm` | https://registry.npmjs.org/ |
| `yarn` | `yarn` | https://registry.yarnpkg.com/ |
| `yarn` | `npmmirror` | https://registry.npmmirror.com/ |
| `yarn` | `taobao` | https://registry.npmmirror.com/ |
| `yarn` | `tencent` | https://mirrors.tencent.com/npm/ |
| `yarn` | `huawei` | https://repo.huaweicloud.com/repository/npm/ |
| `bun` | `npm` | https://registry.npmjs.org/ |
| `bun` | `yarn` | https://registry.yarnpkg.com/ |
| `bun` | `npmmirror` | https://registry.npmmirror.com/ |
| `bun` | `taobao` | https://registry.npmmirror.com/ |
| `bun` | `tencent` | https://mirrors.tencent.com/npm/ |
| `bun` | `huawei` | https://repo.huaweicloud.com/repository/npm/ |
| `go` | `golangcn` | https://proxy.golang.com.cn,direct |
| `go` | `goproxy` | https://goproxy.cn,direct |
| `go` | `goproxyio` | https://goproxy.io,direct |
| `go` | `aliyun` | https://mirrors.aliyun.com/goproxy/,direct |
| `go` | `tencent` | https://mirrors.tencent.com/go,direct |
| `pip` | `tuna` | https://pypi.tuna.tsinghua.edu.cn/simple |
| `pip` | `ustc` | https://mirrors.ustc.edu.cn/pypi/simple |
| `pip` | `aliyun` | https://mirrors.aliyun.com/pypi/simple/ |
| `pip` | `bfsu` | https://mirrors.bfsu.edu.cn/pypi/web/simple |
| `pip` | `tencent` | https://mirrors.tencent.com/pypi/simple/ |
| `pip` | `sjtu` | https://mirror.sjtu.edu.cn/pypi/web/simple/ |
| `pip` | `zju` | https://mirrors.zju.edu.cn/pypi/web/simple/ |
| `pip` | `huawei` | https://repo.huaweicloud.com/repository/pypi/simple/ |
| `pip` | `volcengine` | https://mirrors.volces.com/pypi/simple/ |
| `pip` | `pku` | https://mirrors.pku.edu.cn/pypi/web/simple/ |
| `uv` | `tuna` | https://pypi.tuna.tsinghua.edu.cn/simple |
| `uv` | `ustc` | https://mirrors.ustc.edu.cn/pypi/simple |
| `uv` | `aliyun` | https://mirrors.aliyun.com/pypi/simple/ |
| `uv` | `bfsu` | https://mirrors.bfsu.edu.cn/pypi/web/simple |
| `uv` | `tencent` | https://mirrors.tencent.com/pypi/simple/ |
| `uv` | `sjtu` | https://mirror.sjtu.edu.cn/pypi/web/simple/ |
| `uv` | `zju` | https://mirrors.zju.edu.cn/pypi/web/simple/ |
| `uv` | `huawei` | https://repo.huaweicloud.com/repository/pypi/simple/ |
| `uv` | `volcengine` | https://mirrors.volces.com/pypi/simple/ |
| `uv` | `pku` | https://mirrors.pku.edu.cn/pypi/web/simple/ |
| `pdm` | `tuna` | https://pypi.tuna.tsinghua.edu.cn/simple |
| `pdm` | `ustc` | https://mirrors.ustc.edu.cn/pypi/simple |
| `pdm` | `aliyun` | https://mirrors.aliyun.com/pypi/simple/ |
| `pdm` | `bfsu` | https://mirrors.bfsu.edu.cn/pypi/web/simple |
| `pdm` | `tencent` | https://mirrors.tencent.com/pypi/simple/ |
| `pdm` | `sjtu` | https://mirror.sjtu.edu.cn/pypi/web/simple/ |
| `pdm` | `zju` | https://mirrors.zju.edu.cn/pypi/web/simple/ |
| `pdm` | `huawei` | https://repo.huaweicloud.com/repository/pypi/simple/ |
| `pdm` | `volcengine` | https://mirrors.volces.com/pypi/simple/ |
| `pdm` | `pku` | https://mirrors.pku.edu.cn/pypi/web/simple/ |
| `poetry` | `tuna` | https://pypi.tuna.tsinghua.edu.cn/simple |
| `poetry` | `ustc` | https://mirrors.ustc.edu.cn/pypi/simple |
| `poetry` | `aliyun` | https://mirrors.aliyun.com/pypi/simple/ |
| `poetry` | `bfsu` | https://mirrors.bfsu.edu.cn/pypi/web/simple |
| `poetry` | `tencent` | https://mirrors.tencent.com/pypi/simple/ |
| `poetry` | `sjtu` | https://mirror.sjtu.edu.cn/pypi/web/simple/ |
| `poetry` | `zju` | https://mirrors.zju.edu.cn/pypi/web/simple/ |
| `poetry` | `huawei` | https://repo.huaweicloud.com/repository/pypi/simple/ |
| `poetry` | `volcengine` | https://mirrors.volces.com/pypi/simple/ |
| `poetry` | `pku` | https://mirrors.pku.edu.cn/pypi/web/simple/ |
| `composer` | `aliyun` | https://mirrors.aliyun.com/composer/ |
| `composer` | `huawei` | https://repo.huaweicloud.com/repository/php/ |
| `gem` | `aliyun` | https://mirrors.aliyun.com/rubygems/ |
| `gem` | `ustc` | https://mirrors.ustc.edu.cn/rubygems/ |
| `gem` | `tencent` | https://mirrors.cloud.tencent.com/rubygems/ |
| `gem` | `huawei` | https://repo.huaweicloud.com/repository/rubygems/ |
| `gem` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/rubygems/ |
| `gem` | `nju` | https://mirror.nju.edu.cn/rubygems/ |
| `bundle` | `aliyun` | https://mirrors.aliyun.com/rubygems/ |
| `bundle` | `ustc` | https://mirrors.ustc.edu.cn/rubygems/ |
| `bundle` | `tencent` | https://mirrors.cloud.tencent.com/rubygems/ |
| `bundle` | `huawei` | https://repo.huaweicloud.com/repository/rubygems/ |
| `bundle` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/rubygems/ |
| `bundle` | `nju` | https://mirror.nju.edu.cn/rubygems/ |
| `maven` | `aliyun` | https://maven.aliyun.com/repository/public |
| `maven` | `tencent` | https://mirrors.tencent.com/nexus/repository/maven-public/ |
| `maven` | `huawei` | https://repo.huaweicloud.com/repository/maven/ |
| `maven` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/maven-central/ |
| `maven` | `ustc` | https://maven.proxy.ustclug.org/maven2/ |
| `gradle` | `aliyun` | https://maven.aliyun.com/repository/public |
| `gradle` | `tencent` | https://mirrors.tencent.com/nexus/repository/maven-public/ |
| `gradle` | `huawei` | https://repo.huaweicloud.com/repository/maven/ |
| `gradle` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/maven-central/ |
| `gradle` | `ustc` | https://maven.proxy.ustclug.org/maven2/ |
| `sbt` | `aliyun` | https://maven.aliyun.com/repository/public |
| `sbt` | `tencent` | https://mirrors.tencent.com/nexus/repository/maven-public/ |
| `sbt` | `huawei` | https://repo.huaweicloud.com/repository/maven/ |
| `sbt` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/maven-central/ |
| `sbt` | `ustc` | https://maven.proxy.ustclug.org/maven2/ |
| `cargo` | `rsproxy` | https://rsproxy.cn/index/ |
| `cargo` | `ustc` | https://mirrors.ustc.edu.cn/crates.io-index/ |
| `cargo` | `aliyun` | https://mirrors.aliyun.com/crates.io-index/ |
| `cargo` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/ |
| `docker` | `daocloud` | https://docker.m.daocloud.io |
| `buildkit` | `daocloud` | https://docker.m.daocloud.io |
| `containerd` | `daocloud` | https://docker.m.daocloud.io |
| `podman` | `daocloud` | https://docker.m.daocloud.io |
| `conda` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/anaconda |
| `conda` | `ustc` | https://mirrors.ustc.edu.cn/anaconda |
| `conda` | `nju` | https://mirrors.nju.edu.cn/anaconda |
| `conda` | `huawei` | https://repo.huaweicloud.com/repository/conda |
| `nuget` | `huawei` | https://repo.huaweicloud.com/repository/nuget/ |
| `dart` | `sjtu` | https://mirror.sjtu.edu.cn/dart-pub |
| `dart` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/dart-pub |
| `dart` | `flutter-io` | https://pub.flutter-io.cn |
| `flutter` | `sjtu` | https://mirror.sjtu.edu.cn |
| `flutter` | `flutter-io` | https://storage.flutter-io.cn |
| `cran` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/CRAN |
| `cran` | `ustc` | https://mirrors.ustc.edu.cn/CRAN |
| `huggingface` | `hf-mirror` | https://hf-mirror.com |
| `apt` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/debian |
| `apt` | `ustc` | https://mirrors.ustc.edu.cn/debian |
| `apt` | `aliyun` | https://mirrors.aliyun.com/debian |
| `apt` | `tencent` | https://mirrors.cloud.tencent.com/debian |
| `apt` | `huawei` | https://repo.huaweicloud.com/debian |
| `apk` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/alpine |
| `apk` | `ustc` | https://mirrors.ustc.edu.cn/alpine |
| `apk` | `aliyun` | https://mirrors.aliyun.com/alpine |
| `apk` | `tencent` | https://mirrors.cloud.tencent.com/alpine |
| `apk` | `huawei` | https://repo.huaweicloud.com/alpine |
| `brew` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn |
| `rustup` | `rsproxy` | https://rsproxy.cn |
| `rustup` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/rustup |
| `rustup` | `ustc` | https://mirrors.ustc.edu.cn/rust-static |
| `rustup` | `sjtu` | https://mirror.sjtu.edu.cn/rust-static |
| `rustup` | `zju` | https://mirrors.zju.edu.cn/rustup |
| `cpan` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/CPAN |
| `cpan` | `ustc` | https://mirrors.ustc.edu.cn/CPAN/ |
| `winget` | `ustc` | https://mirrors.ustc.edu.cn/winget-source |
| `nvm` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/ |
| `nvm` | `ustc` | https://mirrors.ustc.edu.cn/node/ |
| `nvm` | `aliyun` | https://mirrors.aliyun.com/nodejs-release/ |
| `nvm` | `tencent` | https://mirrors.cloud.tencent.com/nodejs-release/ |
| `nvm` | `huawei` | https://repo.huaweicloud.com/nodejs/ |
| `clojure` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/clojars/ |
| `haskell` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/hackage/ |
| `haskell` | `ustc` | https://mirrors.ustc.edu.cn/hackage/ |
| `cabal` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/hackage/ |
| `cabal` | `ustc` | https://mirrors.ustc.edu.cn/hackage/ |
| `stack` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/hackage/ |
| `stack` | `ustc` | https://mirrors.ustc.edu.cn/hackage/ |
| `cocoapods` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/git/CocoaPods/Specs.git |
| `flathub` | `ustc` | https://mirrors.ustc.edu.cn/flathub |
| `nix` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store |
| `nix` | `ustc` | https://mirrors.ustc.edu.cn/nix-channels/store |
| `nix` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/nix-channels/store |
| `emacs` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/elpa/gnu/ |
| `emacs` | `ustc` | https://mirrors.ustc.edu.cn/elpa/gnu/ |
| `emacs` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/emacs-elpa/gnu/ |
| `tex` | `tuna` | https://mirrors.tuna.tsinghua.edu.cn/CTAN/systems/texlive/tlnet/ |
| `tex` | `ustc` | https://mirrors.ustc.edu.cn/CTAN/systems/texlive/tlnet/ |
| `tex` | `sjtu` | https://mirrors.sjtug.sjtu.edu.cn/ctan/systems/texlive/tlnet/ |

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