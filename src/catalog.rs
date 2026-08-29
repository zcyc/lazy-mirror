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

const NODE: &[MirrorSpec] = &[
    MirrorSpec {
        name: "npmmirror",
        url: "https://registry.npmmirror.com/",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/npm/",
    },
];
const GO: &[MirrorSpec] = &[
    MirrorSpec {
        name: "goproxy",
        url: "https://goproxy.cn,direct",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/goproxy/",
    },
    MirrorSpec {
        name: "goproxyio",
        url: "https://goproxy.io,direct",
    },
];
const PYPI: &[MirrorSpec] = &[
    MirrorSpec {
        name: "tuna",
        url: "https://pypi.tuna.tsinghua.edu.cn/simple",
    },
    MirrorSpec {
        name: "ustc",
        url: "https://pypi.mirrors.ustc.edu.cn/simple",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/pypi/simple/",
    },
    MirrorSpec {
        name: "bfsu",
        url: "https://mirrors.bfsu.edu.cn/pypi/web/simple",
    },
];
const COMPOSER: &[MirrorSpec] = &[MirrorSpec {
    name: "aliyun",
    url: "https://mirrors.aliyun.com/composer/",
}];
const RUBYGEMS: &[MirrorSpec] = &[
    MirrorSpec {
        name: "ruby-china",
        url: "https://gems.ruby-china.com",
    },
    MirrorSpec {
        name: "aliyun",
        url: "https://mirrors.aliyun.com/rubygems/",
    },
];
const JAVA: &[MirrorSpec] = &[
    MirrorSpec {
        name: "aliyun",
        url: "https://maven.aliyun.com/repository/public",
    },
    MirrorSpec {
        name: "tencent",
        url: "https://mirrors.cloud.tencent.com/repository/maven-public/",
    },
    MirrorSpec {
        name: "huawei",
        url: "https://repo.huaweicloud.com/repository/maven/",
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
];
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
        name: "tuna",
        url: "https://mirrors.tuna.tsinghua.edu.cn/flutter",
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
];
const APK: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn/alpine",
}];
const HOMEBREW: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn",
}];
const RUSTUP: &[MirrorSpec] = &[MirrorSpec {
    name: "rsproxy",
    url: "https://rsproxy.cn",
}];
const JULIA: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn/julia",
}];
const CPAN: &[MirrorSpec] = &[MirrorSpec {
    name: "tuna",
    url: "https://mirrors.tuna.tsinghua.edu.cn/CPAN",
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
        name: "conda",
        aliases: &["mamba", "anaconda"],
        mirrors: CONDA,
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
        mirrors: EMPTY,
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
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "luarocks",
        aliases: &["lua"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "clojure",
        aliases: &["clojars"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "haskell",
        aliases: &["hackage"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "cabal",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "stack",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "ocaml",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "cocoapods",
        aliases: &["cocoa", "pod"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "flathub",
        aliases: &["flatpak"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "nix",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "guix",
        aliases: &[],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "emacs",
        aliases: &["elpa"],
        mirrors: EMPTY,
    },
    TargetSpec {
        name: "tex",
        aliases: &["ctan", "latex", "texlive", "miktex"],
        mirrors: EMPTY,
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
    TargetSpec {
        name: "nuget",
        aliases: &["dotnet"],
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
    if is_url(selection) {
        return Ok(selection.to_owned());
    }
    if let Some(url) = config.mirror(selection) {
        return Ok(url.to_owned());
    }
    spec.mirrors
        .iter()
        .find(|mirror| mirror.name == selection)
        .map(|mirror| mirror.url.to_owned())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unknown mirror {selection} for {target}; use lm list {target} or a URL"),
            )
        })
}

pub fn builtin_mirrors(target: &str) -> io::Result<&'static [MirrorSpec]> {
    find(target)
        .map(|target| target.mirrors)
        .ok_or_else(|| invalid_target(target))
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
            resolve("docker", Some("first"), &config).unwrap(),
            "https://docker.m.daocloud.io"
        );
        fs::remove_file(path).unwrap();
    }
}
