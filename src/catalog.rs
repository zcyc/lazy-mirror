use std::collections::{BTreeMap, BTreeSet};
use std::io;

use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub struct MirrorSpec {
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone, Copy)]
pub struct TargetSpec {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub mirrors: &'static [MirrorSpec],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeResponse {
    Any,
    JsonObject,
    JsonArray,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProbeSpec {
    pub suffix: &'static str,
    pub response: ProbeResponse,
}

const NODE: &[MirrorSpec] = &[
    MirrorSpec {
        name: "npm",
        url: "https://registry.npmjs.org/",
    },
    MirrorSpec {
        name: "yarn",
        url: "https://registry.yarnpkg.com/",
    },
    MirrorSpec {
        name: "npmmirror",
        url: "https://registry.npmmirror.com/",
    },
    MirrorSpec {
        name: "taobao",
        url: "https://registry.npmmirror.com/",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.tencent.com/npm/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/npm/",
    },
];
const NVM: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/nodejs-release/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/node/",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/nodejs-release/",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/nodejs-release/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/nodejs/",
    },
];
const GO: &[MirrorSpec] = &[
    MirrorSpec {
        name: "golangcn",
        url: "https://proxy.golang.com.cn,direct",
    },
    MirrorSpec {
        name: "goproxy",
        url: "https://goproxy.cn,direct",
    },
    MirrorSpec {
        name: "goproxyio",
        url: "https://goproxy.io,direct",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.tencent.com/go,direct",
    },
];
const PYPI: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://pypi.tuna.tsinghua.edu.cn/simple",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/pypi/simple",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/pypi/simple/",
    },
    MirrorSpec {
        name: "bfsu",
        url: "https://mirrors.bfsu.edu.cn/pypi/web/simple",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.tencent.com/pypi/simple/",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirror.sjtu.edu.cn/pypi/web/simple/",
    },
    MirrorSpec {
        name: "zju",
        url: "https://mirrors.zju.edu.cn/pypi/web/simple/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/pypi/simple/",
    },
    MirrorSpec {
        name: "volcengine",
        url: "https://mirrors.volces.com/pypi/simple/",
    },
    MirrorSpec {
        name: "pku",
        url: "https://mirrors.pku.edu.cn/pypi/web/simple/",
    },
];
const COMPOSER: &[MirrorSpec] = &[
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/composer/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/php/",
    },
];
const RUBYGEMS: &[MirrorSpec] = &[
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/rubygems/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/rubygems/",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/rubygems/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/rubygems/",
    },
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/rubygems/",
    },
    MirrorSpec {
        name: "nju",
        url: "https://mirror.nju.edu.cn/rubygems/",
    },
];
const JAVA: &[MirrorSpec] = &[
    MirrorSpec {
        name: "aliyun",
        url: "https://maven.aliyun.com/repository/public",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.tencent.com/nexus/repository/maven-public/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/maven/",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirrors.sjtug.sjtu.edu.cn/maven-central/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://maven.proxy.ustclug.org/maven2/",
    },
];
const CARGO: &[MirrorSpec] = &[
    MirrorSpec {
        name: "rsproxy",
        url: "https://rsproxy.cn/index/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/crates.io-index/",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/crates.io-index/",
    },
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/crates.io-index/",
    },
];
const DOCKER: &[MirrorSpec] = &[MirrorSpec {
    name: "daocloud",
    url: "https://docker.m.daocloud.io",
}];
const CONDA: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/anaconda",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/anaconda",
    },
    MirrorSpec {
        name: "nju",
        url: "https://mirrors.nju.edu.cn/anaconda",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/conda",
    },
];
const NUGET: &[MirrorSpec] = &[MirrorSpec {
    name: "huawei",
    url: "https://repo.huaweicloud.com/repository/nuget/",
}];
const DART: &[MirrorSpec] = &[
    MirrorSpec {
        name: "sjtu",
        url: "https://mirror.sjtu.edu.cn/dart-pub",
    },
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/dart-pub",
    },
    MirrorSpec {
        name: "flutter-io",
        url: "https://pub.flutter-io.cn",
    },
];
const FLUTTER: &[MirrorSpec] = &[
    MirrorSpec {
        name: "sjtu",
        url: "https://mirror.sjtu.edu.cn",
    },
    MirrorSpec {
        name: "flutter-io",
        url: "https://storage.flutter-io.cn",
    },
];
const CRAN: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/CRAN",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/CRAN",
    },
];
const HUGGINGFACE: &[MirrorSpec] = &[MirrorSpec {
    name: "hf-mirror",
    url: "https://hf-mirror.com",
}];
const APT: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/debian",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/debian",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/debian",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/debian",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/debian",
    },
];
const APK: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/alpine",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/alpine",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/alpine",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/alpine",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/alpine",
    },
];
const HOMEBREW: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn",
}];
const RUSTUP: &[MirrorSpec] = &[
    MirrorSpec {
        name: "rsproxy",
        url: "https://rsproxy.cn",
    },
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/rustup",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/rust-static",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirror.sjtu.edu.cn/rust-static",
    },
    MirrorSpec {
        name: "zju",
        url: "https://mirrors.zju.edu.cn/rustup",
    },
];
const JULIA: &[MirrorSpec] = &[];
const CPAN: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/CPAN",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/CPAN/",
    },
];
const HASKELL: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/hackage/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/hackage/",
    },
];
const CLOJURE: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn/clojars/",
}];
const COCOAPODS: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn/git/CocoaPods/Specs.git",
}];
const FLATHUB: &[MirrorSpec] = &[MirrorSpec {
    name: "ustc",
    url: "https://mirrors.ustc.edu.cn/flathub",
}];
const NIX: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/nix-channels/store",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/nix-channels/store",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirrors.sjtug.sjtu.edu.cn/nix-channels/store",
    },
];
const EMACS: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/elpa/gnu/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/elpa/gnu/",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirrors.sjtug.sjtu.edu.cn/emacs-elpa/gnu/",
    },
];
const TEX: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/CTAN/systems/texlive/tlnet/",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://mirrors.ustc.edu.cn/CTAN/systems/texlive/tlnet/",
    },
    MirrorSpec {
        name: "sjtu",
        url: "https://mirrors.sjtug.sjtu.edu.cn/ctan/systems/texlive/tlnet/",
    },
];
const WINGET: &[MirrorSpec] = &[MirrorSpec {
    name: "ustc",
    url: "https://mirrors.ustc.edu.cn/winget-source",
}];
const EMPTY: &[MirrorSpec] = &[];

const TARGETS: &[TargetSpec] = &[
    TargetSpec {
        name: "npm",
        aliases: &["node", "nodejs"],
        mirrors: NODE,
    },
    TargetSpec {
        name: "pnpm",
        aliases: &[],
        mirrors: NODE,
    },
    TargetSpec {
        name: "yarn",
        aliases: &[],
        mirrors: NODE,
    },
    TargetSpec {
        name: "bun",
        aliases: &[],
        mirrors: NODE,
    },
    TargetSpec {
        name: "go",
        aliases: &[],
        mirrors: GO,
    },
    TargetSpec {
        name: "pip",
        aliases: &["pip3", "python", "py", "pypi"],
        mirrors: PYPI,
    },
    TargetSpec {
        name: "uv",
        aliases: &[],
        mirrors: PYPI,
    },
    TargetSpec {
        name: "pdm",
        aliases: &[],
        mirrors: PYPI,
    },
    TargetSpec {
        name: "poetry",
        aliases: &[],
        mirrors: PYPI,
    },
    TargetSpec {
        name: "composer",
        aliases: &["php"],
        mirrors: COMPOSER,
    },
    TargetSpec {
        name: "gem",
        aliases: &["ruby"],
        mirrors: RUBYGEMS,
    },
    TargetSpec {
        name: "bundle",
        aliases: &["bundler"],
        mirrors: RUBYGEMS,
    },
    TargetSpec {
        name: "maven",
        aliases: &["java", "mvn", "maven-daemon", "mvnd"],
        mirrors: JAVA,
    },
    TargetSpec {
        name: "gradle",
        aliases: &[],
        mirrors: JAVA,
    },
    TargetSpec {
        name: "sbt",
        aliases: &[],
        mirrors: JAVA,
    },
    TargetSpec {
        name: "cargo",
        aliases: &["rust", "crate"],
        mirrors: CARGO,
    },
    TargetSpec {
        name: "docker",
        aliases: &["dockerhub"],
        mirrors: DOCKER,
    },
    TargetSpec {
        name: "buildkit",
        aliases: &["docker-buildkit", "buildx"],
        mirrors: DOCKER,
    },
    TargetSpec {
        name: "containerd",
        aliases: &["nerdctl"],
        mirrors: DOCKER,
    },
    TargetSpec {
        name: "podman",
        aliases: &[],
        mirrors: DOCKER,
    },
    TargetSpec {
        name: "helm",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "conda",
        aliases: &["mamba", "anaconda"],
        mirrors: CONDA,
    },
    TargetSpec {
        name: "nuget",
        aliases: &["dotnet"],
        mirrors: NUGET,
    },
    TargetSpec {
        name: "dart",
        aliases: &["pub"],
        mirrors: DART,
    },
    TargetSpec {
        name: "flutter",
        aliases: &[],
        mirrors: FLUTTER,
    },
    TargetSpec {
        name: "cran",
        aliases: &["r"],
        mirrors: CRAN,
    },
    TargetSpec {
        name: "huggingface",
        aliases: &["hf", "huggingface-hub"],
        mirrors: HUGGINGFACE,
    },
    TargetSpec {
        name: "apt",
        aliases: &["debian", "ubuntu"],
        mirrors: APT,
    },
    TargetSpec {
        name: "apk",
        aliases: &["alpine"],
        mirrors: APK,
    },
    TargetSpec {
        name: "brew",
        aliases: &["homebrew"],
        mirrors: HOMEBREW,
    },
    TargetSpec {
        name: "rustup",
        aliases: &[],
        mirrors: RUSTUP,
    },
    TargetSpec {
        name: "hex",
        aliases: &["mix"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "julia",
        aliases: &[],
        mirrors: JULIA,
    },
    TargetSpec {
        name: "cpan",
        aliases: &["perl"],
        mirrors: CPAN,
    },
    TargetSpec {
        name: "winget",
        aliases: &[],
        mirrors: WINGET,
    },
    TargetSpec {
        name: "opam",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "rye",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "nvm",
        aliases: &[],
        mirrors: NVM,
    },
    TargetSpec {
        name: "luarocks",
        aliases: &["lua"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "clojure",
        aliases: &["clojars"],
        mirrors: CLOJURE,
    },
    TargetSpec {
        name: "haskell",
        aliases: &["hackage"],
        mirrors: HASKELL,
    },
    TargetSpec {
        name: "cabal",
        aliases: &[],
        mirrors: HASKELL,
    },
    TargetSpec {
        name: "stack",
        aliases: &[],
        mirrors: HASKELL,
    },
    TargetSpec {
        name: "ocaml",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "cocoapods",
        aliases: &["cocoa", "pod"],
        mirrors: COCOAPODS,
    },
    TargetSpec {
        name: "flathub",
        aliases: &["flatpak"],
        mirrors: FLATHUB,
    },
    TargetSpec {
        name: "nix",
        aliases: &[],
        mirrors: NIX,
    },
    TargetSpec {
        name: "guix",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "emacs",
        aliases: &["elpa"],
        mirrors: EMACS,
    },
    TargetSpec {
        name: "tex",
        aliases: &["ctan", "latex", "texlive", "miktex"],
        mirrors: TEX,
    },
    TargetSpec {
        name: "linuxmint",
        aliases: &["mint", "zorinos"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "fedora",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "opensuse",
        aliases: &["suse"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "kali",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "arch",
        aliases: &["archlinux"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "archlinuxcn",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "manjaro",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "gentoo",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "rocky",
        aliases: &["rockylinux"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "alma",
        aliases: &["almalinux"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "voidlinux",
        aliases: &["void"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "solus",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "ros",
        aliases: &["ros2"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "trisquel",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "linuxlite",
        aliases: &["lite"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "raspi",
        aliases: &["raspberrypi"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "armbian",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "openwrt",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "openeuler",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "openanolis",
        aliases: &["anolis"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "openkylin",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "deepin",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "msys2",
        aliases: &["msys"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "termux",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "freebsd",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "openbsd",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "netbsd",
        aliases: &[],
        mirrors: EMPTY,
    },
];

pub fn targets() -> &'static [TargetSpec] {
    TARGETS
}

pub fn find(name: &str) -> Option<&'static TargetSpec> {
    TARGETS
        .iter()
        .find(|target| target.name == name || target.aliases.contains(&name))
}

pub fn probe_spec(target: &str) -> ProbeSpec {
    let (suffix, response) = match target {
        "apt" | "ros" => ("/dists/{distribution}/Release", ProbeResponse::Any),
        "npm" | "pnpm" | "yarn" | "bun" => ("/-/ping", ProbeResponse::JsonObject),
        "pip" | "uv" | "pdm" | "poetry" => ("/simple/pip/", ProbeResponse::Any),
        "go" => ("/github.com/golang/go/@v/list", ProbeResponse::Any),
        "docker" | "buildkit" | "containerd" | "podman" => ("/v2/", ProbeResponse::Any),
        "composer" => ("/packages.json", ProbeResponse::Any),
        "gem" | "bundle" => ("/specs.4.8.gz", ProbeResponse::Any),
        "maven" | "gradle" | "sbt" => (
            "/org/apache/maven/maven-core/maven-metadata.xml",
            ProbeResponse::Any,
        ),
        "cargo" => ("/config.json", ProbeResponse::JsonObject),
        "helm" => ("/index.yaml", ProbeResponse::Any),
        "conda" => ("/pkgs/main/linux-64/repodata.json", ProbeResponse::Any),
        "dart" => ("/api/packages/http", ProbeResponse::Any),
        "flutter" => (
            "/flutter_infra_release/releases/stable/linux/flutter_linux_3.35.5-stable.tar.xz",
            ProbeResponse::Any,
        ),
        "cran" => ("/src/contrib/PACKAGES", ProbeResponse::Any),
        "huggingface" => ("/api/models?limit=1", ProbeResponse::JsonArray),
        "nuget" => ("/v3/index.json", ProbeResponse::JsonObject),
        "apk" => (
            "/latest-stable/main/x86_64/APKINDEX.tar.gz",
            ProbeResponse::Any,
        ),
        "rustup" => ("/dist/channel-rust-stable.toml", ProbeResponse::Any),
        "nvm" => ("/index.tab", ProbeResponse::Any),
        "cpan" => ("/modules/02packages.details.txt.gz", ProbeResponse::Any),
        "luarocks" => ("/manifest", ProbeResponse::Any),
        "hackage" | "cabal" | "stack" => ("/01-index.tar.gz", ProbeResponse::Any),
        "clojure" => (
            "/ring/ring-core/1.12.2/ring-core-1.12.2.pom",
            ProbeResponse::Any,
        ),
        "cocoapods" => ("/info/refs?service=git-upload-pack", ProbeResponse::Any),
        "flathub" => ("/summary.idx", ProbeResponse::Any),
        "nix" => ("/nix-cache-info", ProbeResponse::Any),
        "emacs" => ("/archive-contents", ProbeResponse::Any),
        "tex" => ("/tlpkg/texlive.tlpdb", ProbeResponse::Any),
        _ => ("", ProbeResponse::Any),
    };
    ProbeSpec { suffix, response }
}

pub fn resolve(target: &str, selector: Option<&str>, config: &Config) -> io::Result<String> {
    let spec = find(target).ok_or_else(|| invalid_target(target))?;
    let selection = selector.or_else(|| config.default_for(spec.name));
    let Some(selection) = selection else {
        return spec.mirrors.first().map_or_else(
            || {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{target} requires a mirror name or URL"),
                ))
            },
            |mirror| Ok(mirror.url.to_owned()),
        );
    };
    if selection == "first" {
        return spec.mirrors.first().map_or_else(
            || {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("{target} has no built-in mirror"),
                ))
            },
            |mirror| Ok(mirror.url.to_owned()),
        );
    }
    if crate::config::is_selection_url(spec.name, selection) {
        return Ok(selection.to_owned());
    }
    if let Some(url) = config.mirror(selection) {
        if crate::config::is_selection_url(spec.name, url) {
            return Ok(url.to_owned());
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "mirror {} is not valid for {target}",
                crate::config::redact_selection(selection)
            ),
        ));
    }
    spec.mirrors
        .iter()
        .find(|mirror| mirror.name == selection)
        .map(|mirror| mirror.url.to_owned())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "unknown mirror {} for {target}; use lm list {target} or a URL",
                    crate::config::redact_selection(selection)
                ),
            )
        })
}

pub fn builtin_mirrors(target: &str) -> io::Result<&'static [MirrorSpec]> {
    find(target)
        .map(|target| target.mirrors)
        .ok_or_else(|| invalid_target(target))
}

pub fn lint() -> io::Result<()> {
    let mut selectors = BTreeMap::new();
    for target in TARGETS {
        if selectors.insert(target.name, target.name).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate catalog target {}", target.name),
            ));
        }
        for alias in target.aliases {
            if selectors.insert(*alias, target.name).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate catalog selector {alias}"),
                ));
            }
        }
        let mut mirrors = BTreeSet::new();
        for mirror in target.mirrors {
            if mirror.name.is_empty() || !mirrors.insert(mirror.name) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate or empty mirror name in {}", target.name),
                ));
            }
            if !is_url(mirror.url) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid mirror URL {} for {}", mirror.url, target.name),
                ));
            }
        }
    }
    Ok(())
}

fn is_url(value: &str) -> bool {
    crate::config::is_url(value)
}

fn invalid_target(target: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("unsupported target: {target}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn config_url_overrides_builtin_selection() {
        let path = std::env::temp_dir().join(format!("lm-catalog-{}.toml", std::process::id()));
        fs::write(
            &path,
            "[mirrors]\ncorp = { url = \"https://mirror.example/simple\" }\n[defaults]\npip = \"corp\"\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(
            resolve("pip", None, &config).unwrap(),
            "https://mirror.example/simple"
        );
        assert_eq!(
            resolve("pip", Some("https://override.example/simple"), &config).unwrap(),
            "https://override.example/simple"
        );
        assert_eq!(
            resolve("huggingface", Some("hf-mirror"), &config).unwrap(),
            "https://hf-mirror.com"
        );
        assert_eq!(
            resolve(
                "cargo",
                Some("sparse+https://mirror.example/index/"),
                &config
            )
            .unwrap(),
            "sparse+https://mirror.example/index/"
        );
        assert_eq!(
            resolve("docker", Some("first"), &config).unwrap(),
            "https://docker.m.daocloud.io"
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn built_in_catalog_is_valid() {
        lint().unwrap();
    }

    #[test]
    fn probe_specs_keep_protocol_contracts_with_the_catalog() {
        assert_eq!(
            probe_spec("huggingface"),
            ProbeSpec {
                suffix: "/api/models?limit=1",
                response: ProbeResponse::JsonArray,
            }
        );
        assert_eq!(
            probe_spec("pip"),
            ProbeSpec {
                suffix: "/simple/pip/",
                response: ProbeResponse::Any,
            }
        );
        assert_eq!(probe_spec("unknown").suffix, "");
    }
}
