//! Fake-binary test harness.
#![allow(dead_code)]
#![allow(clippy::new_ret_no_self)]
//!
//! `FakeBin` writes a minimal shell script to a `TempDir` that prints fixed output and
//! exits with a configured code. Prepend `dir()` to `PATH` before calling any integration
//! under test.
//!
//! Usage:
//! ```ignore
//! let fake = FakeBin::new("doob").stdout(r#"{"count":1,"todos":[]}"#).build();
//! let path = format!("{}:{}", fake.dir(), std::env::var("PATH").unwrap_or_default());
//! std::env::set_var("PATH", &path);
//! // ... call integration code ...
//! ```

use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use tempfile::TempDir;

pub struct FakeBin {
    _dir: TempDir,
    dir_path: PathBuf,
}

pub struct FakeBinBuilder {
    name: String,
    stdout: String,
    stderr: String,
    exit_code: i32,
    /// If set, echo argv as JSON array instead of fixed stdout.
    echo_argv: bool,
}

impl FakeBinBuilder {
    pub fn stdout(mut self, s: &str) -> Self {
        self.stdout = s.to_string();
        self
    }

    pub fn stderr(mut self, s: &str) -> Self {
        self.stderr = s.to_string();
        self
    }

    pub fn exit_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    pub fn echo_argv(mut self) -> Self {
        self.echo_argv = true;
        self
    }

    pub fn build(self) -> FakeBin {
        let dir = TempDir::new().expect("tempdir");
        let bin_path = dir.path().join(&self.name);

        let script = if self.echo_argv {
            // Print argv as a JSON array using echo to avoid printf format-string issues.
            format!(
                "#!/bin/sh\nout='['; sep=''; for a in \"$@\"; do out=\"${{out}}${{sep}}\\\"${{a}}\\\"\"; sep=','; done; out=\"${{out}}]\"; echo \"$out\"\nexit {}\n",
                self.exit_code
            )
        } else {
            let stdout_escaped = self.stdout.replace('\'', "'\\''");
            let stderr_escaped = self.stderr.replace('\'', "'\\''");
            format!(
                "#!/bin/sh\nprintf '%s' '{}'\n>&2 printf '%s' '{}'\nexit {}\n",
                stdout_escaped, stderr_escaped, self.exit_code
            )
        };

        std::fs::write(&bin_path, script).expect("write fake bin");
        std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake bin");

        let dir_path = dir.path().to_path_buf();
        FakeBin {
            _dir: dir,
            dir_path,
        }
    }
}

impl FakeBin {
    pub fn new(name: &str) -> FakeBinBuilder {
        FakeBinBuilder {
            name: name.to_string(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            echo_argv: false,
        }
    }

    /// Directory containing the fake binary — prepend to PATH.
    pub fn dir(&self) -> &str {
        self.dir_path.to_str().expect("utf8 path")
    }

    /// Convenience: build a PATH string with this dir prepended.
    pub fn path_with(&self) -> String {
        let existing = std::env::var("PATH").unwrap_or_default();
        format!("{}:{}", self.dir(), existing)
    }
}
