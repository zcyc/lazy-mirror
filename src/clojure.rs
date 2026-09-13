use std::io;

use crate::config::Scope;
use crate::shared::{config_path, file_status};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("clojure", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!(
            ";; managed by lazy-mirror\n{{:mvn/repos {{:central {{:url \"https://repo.maven.apache.org/maven2/\"}} :clojars {{:url \"{mirror}\"}}}}}}\n"
        ),
        |content| content.starts_with(";; managed by lazy-mirror\n"),
    )
    .map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "{} already contains unmanaged EDN; set LM_CLOJURE_CONFIG to a dedicated file",
                    path.display()
                ),
            )
        } else {
            error
        }
    })
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("clojure", scope)?, |content| {
        content.starts_with(";; managed by lazy-mirror\n")
    })
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("clojure", scope)?;
    file_status("clojure", &path, expected, |content| {
        content
            .split_once(":clojars {:url \"")
            .and_then(|(_, value)| value.split_once('"').map(|(value, _)| value.to_owned()))
    })
}
