//! `godmode init` — global and project-level setup.

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::config::Config;
use crate::doctor::{DoctorReport, EnvironmentProbe};

/// Port: filesystem operations needed by init.
pub trait InitFs {
    /// Returns whether `path` exists as a directory.
    fn dir_exists(&self, path: &Path) -> bool;
    /// Creates `path` and any missing parent directories.
    fn create_dir_all(&self, path: &Path) -> Result<()>;
    /// Writes UTF-8 `contents` to `path`, replacing any existing file.
    fn write_file(&self, path: &Path, contents: &str) -> Result<()>;
    /// Reads the UTF-8 contents of `path`.
    fn read_to_string(&self, path: &Path) -> Result<String>;
    /// Returns whether `path` exists as a file.
    fn file_exists(&self, path: &Path) -> bool;
}

/// What init created.
#[derive(Debug, Clone, serde::Serialize)]
pub struct InitReport {
    /// Whether this run created the global configuration directory.
    pub global_created: bool,
    /// Path to the global configuration directory.
    pub global_path: PathBuf,
    /// Whether this run created the project's godmode context directory.
    pub project_created: bool,
    /// Path to the project context directory, when a Cargo project was found.
    pub project_path: Option<PathBuf>,
    /// Whether this run added `.ctx/` to the project `.gitignore`.
    pub gitignore_updated: bool,
    /// Environment diagnostic results collected after initialization.
    pub doctor: DoctorReport,
}

/// Detect whether `start` or an ancestor contains `Cargo.toml`.
fn find_cargo_root(fs: &dyn InitFs, start: &Path) -> Option<PathBuf> {
    let mut cur = start.to_path_buf();
    loop {
        if fs.file_exists(&cur.join("Cargo.toml")) {
            return Some(cur);
        }
        if !cur.pop() {
            return None;
        }
    }
}

/// Run init with injected filesystem and probe.
pub fn run_init(
    fs: &dyn InitFs,
    probe: &dyn EnvironmentProbe,
    cwd: &Path,
    global_config_dir: &Path,
) -> Result<InitReport> {
    // --- Global setup ---
    let global_created = if !fs.dir_exists(global_config_dir) {
        fs.create_dir_all(global_config_dir)?;
        let cfg = Config::default();
        let toml_str = toml::to_string_pretty(&cfg)?;
        fs.write_file(&global_config_dir.join("config.toml"), &toml_str)?;
        true
    } else {
        false
    };

    // --- Project setup ---
    let cargo_root = find_cargo_root(fs, cwd);
    let mut project_created = false;
    let mut project_path: Option<PathBuf> = None;
    let mut gitignore_updated = false;

    if let Some(ref root) = cargo_root {
        let ctx_dir = root.join(".ctx").join("godmode");
        if !fs.dir_exists(&ctx_dir) {
            fs.create_dir_all(&ctx_dir)?;
            fs.write_file(&ctx_dir.join("tasks.yaml"), "tasks: []\n")?;
            fs.write_file(&ctx_dir.join("trace.jsonl"), "")?;
            fs.write_file(&ctx_dir.join("session.toml"), "[session]\n")?;
            project_created = true;
        }
        project_path = Some(ctx_dir);

        // Update .gitignore
        let gi_path = root.join(".gitignore");
        let entry = ".ctx/";
        let needs_entry = if fs.file_exists(&gi_path) {
            let contents = fs.read_to_string(&gi_path).unwrap_or_default();
            !contents.lines().any(|l| l.trim() == entry)
        } else {
            true
        };
        if needs_entry {
            let existing = if fs.file_exists(&gi_path) {
                fs.read_to_string(&gi_path).unwrap_or_default()
            } else {
                String::new()
            };
            let new_contents = if existing.is_empty() {
                format!("{entry}\n")
            } else if existing.ends_with('\n') {
                format!("{existing}{entry}\n")
            } else {
                format!("{existing}\n{entry}\n")
            };
            fs.write_file(&gi_path, &new_contents)?;
            gitignore_updated = true;
        }
    }

    let doctor = crate::doctor::run_doctor(probe);

    Ok(InitReport {
        global_created,
        global_path: global_config_dir.to_path_buf(),
        project_created,
        project_path,
        gitignore_updated,
        doctor,
    })
}

/// Real filesystem adapter.
pub struct RealFs;

impl InitFs for RealFs {
    fn dir_exists(&self, path: &Path) -> bool {
        path.is_dir()
    }
    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path)?;
        Ok(())
    }
    fn write_file(&self, path: &Path, contents: &str) -> Result<()> {
        std::fs::write(path, contents)?;
        Ok(())
    }
    fn read_to_string(&self, path: &Path) -> Result<String> {
        Ok(std::fs::read_to_string(path)?)
    }
    fn file_exists(&self, path: &Path) -> bool {
        path.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doctor::EnvironmentProbe;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    struct MemFs {
        dirs: Mutex<std::collections::BTreeSet<PathBuf>>,
        files: Mutex<BTreeMap<PathBuf, String>>,
    }

    impl MemFs {
        fn new() -> Self {
            Self {
                dirs: Mutex::new(std::collections::BTreeSet::new()),
                files: Mutex::new(BTreeMap::new()),
            }
        }

        fn seed_file(&self, path: &Path, content: &str) {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), content.into());
        }

        fn seed_dir(&self, path: &Path) {
            self.dirs.lock().unwrap().insert(path.to_path_buf());
        }

        fn get_file(&self, path: &Path) -> Option<String> {
            self.files.lock().unwrap().get(path).cloned()
        }
    }

    impl InitFs for MemFs {
        fn dir_exists(&self, path: &Path) -> bool {
            self.dirs.lock().unwrap().contains(path)
        }
        fn create_dir_all(&self, path: &Path) -> Result<()> {
            self.dirs.lock().unwrap().insert(path.to_path_buf());
            Ok(())
        }
        fn write_file(&self, path: &Path, contents: &str) -> Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), contents.into());
            Ok(())
        }
        fn read_to_string(&self, path: &Path) -> Result<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("not found"))
        }
        fn file_exists(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }
    }

    struct PassProbe;
    impl EnvironmentProbe for PassProbe {
        fn has_tool(&self, _: &str) -> bool {
            true
        }
        fn op_authed(&self) -> bool {
            true
        }
        fn stale_worktrees(&self) -> Vec<String> {
            vec![]
        }
    }

    #[test]
    fn global_init_creates_config() {
        let fs = MemFs::new();
        let report = run_init(
            &fs,
            &PassProbe,
            Path::new("/tmp/noproject"),
            Path::new("/home/user/.config/godmode"),
        )
        .unwrap();
        assert!(report.global_created);
        assert!(!report.project_created);
        assert!(report.project_path.is_none());
        assert!(
            fs.get_file(Path::new("/home/user/.config/godmode/config.toml"))
                .is_some()
        );
    }

    #[test]
    fn global_init_skips_if_exists() {
        let fs = MemFs::new();
        fs.seed_dir(Path::new("/home/user/.config/godmode"));
        let report = run_init(
            &fs,
            &PassProbe,
            Path::new("/tmp"),
            Path::new("/home/user/.config/godmode"),
        )
        .unwrap();
        assert!(!report.global_created);
    }

    #[test]
    fn project_init_creates_ctx_dir() {
        let fs = MemFs::new();
        fs.seed_dir(Path::new("/home/user/.config/godmode"));
        fs.seed_file(
            Path::new("/projects/myapp/Cargo.toml"),
            "[package]\nname = \"myapp\"\n",
        );
        let report = run_init(
            &fs,
            &PassProbe,
            Path::new("/projects/myapp"),
            Path::new("/home/user/.config/godmode"),
        )
        .unwrap();
        assert!(report.project_created);
        assert_eq!(
            report.project_path,
            Some(PathBuf::from("/projects/myapp/.ctx/godmode"))
        );
        assert!(
            fs.get_file(Path::new("/projects/myapp/.ctx/godmode/tasks.yaml"))
                .is_some()
        );
    }

    #[test]
    fn gitignore_updated_when_missing_entry() {
        let fs = MemFs::new();
        fs.seed_dir(Path::new("/cfg"));
        fs.seed_file(Path::new("/p/Cargo.toml"), "[package]\nname=\"x\"\n");
        fs.seed_file(Path::new("/p/.gitignore"), "target/\n");
        let report = run_init(&fs, &PassProbe, Path::new("/p"), Path::new("/cfg")).unwrap();
        assert!(report.gitignore_updated);
        let gi = fs.get_file(Path::new("/p/.gitignore")).unwrap();
        assert!(gi.contains(".ctx/"));
    }

    #[test]
    fn gitignore_not_updated_when_already_present() {
        let fs = MemFs::new();
        fs.seed_dir(Path::new("/cfg"));
        fs.seed_file(Path::new("/p/Cargo.toml"), "[package]\nname=\"x\"\n");
        fs.seed_file(Path::new("/p/.gitignore"), "target/\n.ctx/\n");
        let report = run_init(&fs, &PassProbe, Path::new("/p"), Path::new("/cfg")).unwrap();
        assert!(!report.gitignore_updated);
    }

    #[test]
    fn idempotent_second_run() {
        let fs = MemFs::new();
        fs.seed_file(Path::new("/p/Cargo.toml"), "[package]\nname=\"x\"\n");
        let cfg = Path::new("/cfg");

        let r1 = run_init(&fs, &PassProbe, Path::new("/p"), cfg).unwrap();
        assert!(r1.global_created);
        assert!(r1.project_created);

        let r2 = run_init(&fs, &PassProbe, Path::new("/p"), cfg).unwrap();
        assert!(!r2.global_created);
        assert!(!r2.project_created);
        assert!(!r2.gitignore_updated);
    }

    #[test]
    fn doctor_runs_as_part_of_init() {
        let fs = MemFs::new();
        let report = run_init(&fs, &PassProbe, Path::new("/tmp"), Path::new("/cfg")).unwrap();
        assert!(report.doctor.all_passed);
        assert!(!report.doctor.checks.is_empty());
    }
}
