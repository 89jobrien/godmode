//! Transactional filesystem projection support.

use anyhow::{Context, Result, bail};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};

/// One rendered file in a transactional projection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ProjectionFile {
    file_name: String,
    content: String,
}

impl ProjectionFile {
    /// Create a rendered projection file.
    pub fn new(file_name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            content: content.into(),
        }
    }

    /// Return the destination file name.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Return the rendered file content.
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Write all files as one recoverable filesystem projection.
///
/// Every file is staged before any destination changes. If a later replacement fails, earlier
/// replacements are rolled back to their original contents.
///
/// # Examples
///
/// ```
/// use godmode_core::projection::{ProjectionFile, write_projection};
///
/// # fn main() -> anyhow::Result<()> {
/// let output = tempfile::tempdir()?;
/// let files = [ProjectionFile::new("gm-plan.md", "# Plan\n")];
/// let paths = write_projection(&files, output.path(), false)?;
/// assert_eq!(std::fs::read_to_string(&paths[0])?, "# Plan\n");
/// # Ok(())
/// # }
/// ```
pub fn write_projection(
    files: &[ProjectionFile],
    output_dir: &Path,
    dry_run: bool,
) -> Result<Vec<PathBuf>> {
    write_projection_with_hook(files, output_dir, dry_run, |_, _| Ok(()))
}

struct StagedFile {
    target: PathBuf,
    temporary: PathBuf,
}

struct AppliedFile {
    target: PathBuf,
    backup: Option<PathBuf>,
}

fn write_projection_with_hook<F>(
    files: &[ProjectionFile],
    output_dir: &Path,
    dry_run: bool,
    mut before_replace: F,
) -> Result<Vec<PathBuf>>
where
    F: FnMut(usize, &Path) -> io::Result<()>,
{
    validate_files(files)?;
    let paths = files
        .iter()
        .map(|file| output_dir.join(file.file_name()))
        .collect::<Vec<_>>();
    if dry_run {
        return Ok(paths);
    }

    fs::create_dir_all(output_dir)
        .with_context(|| format!("creating projection directory {}", output_dir.display()))?;
    validate_destinations(&paths)?;
    let staged = stage_files(files, &paths, output_dir)?;
    apply_staged(staged, &mut before_replace)?;
    Ok(paths)
}

fn validate_files(files: &[ProjectionFile]) -> Result<()> {
    let mut names = BTreeSet::new();
    for file in files {
        let path = Path::new(file.file_name());
        let mut components = path.components();
        let valid =
            matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none();
        if !valid {
            bail!("invalid projection file name {:?}", file.file_name());
        }
        if !names.insert(file.file_name()) {
            bail!("duplicate projection file name {:?}", file.file_name());
        }
    }
    Ok(())
}

fn validate_destinations(paths: &[PathBuf]) -> Result<()> {
    for path in paths {
        match fs::symlink_metadata(path) {
            Ok(metadata) if !metadata.file_type().is_file() => {
                bail!(
                    "projection destination is not a regular file: {}",
                    path.display()
                );
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("reading projection destination {}", path.display()));
            }
        }
    }
    Ok(())
}

fn stage_files(
    files: &[ProjectionFile],
    paths: &[PathBuf],
    output_dir: &Path,
) -> Result<Vec<StagedFile>> {
    let mut staged = Vec::with_capacity(files.len());
    for (file, target) in files.iter().zip(paths) {
        match stage_file(file, target, output_dir) {
            Ok(staged_file) => staged.push(staged_file),
            Err(error) => {
                cleanup_temporaries(&staged);
                return Err(error);
            }
        }
    }
    Ok(staged)
}

fn stage_file(file: &ProjectionFile, target: &Path, output_dir: &Path) -> Result<StagedFile> {
    let (temporary, mut handle) = create_temporary(output_dir, file.file_name(), "stage")?;
    let result = (|| -> Result<()> {
        handle
            .write_all(file.content().as_bytes())
            .with_context(|| format!("writing staged projection {}", temporary.display()))?;
        handle
            .sync_all()
            .with_context(|| format!("syncing staged projection {}", temporary.display()))?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(StagedFile {
        target: target.to_path_buf(),
        temporary,
    })
}

fn create_temporary(parent: &Path, file_name: &str, kind: &str) -> Result<(PathBuf, File)> {
    for attempt in 0..100_u32 {
        let path = parent.join(format!(
            ".{file_name}.{}.{}.{}.tmp",
            std::process::id(),
            kind,
            attempt
        ));
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("creating projection temporary {}", path.display()));
            }
        }
    }
    bail!("unable to allocate projection temporary for {file_name}")
}

fn reserve_backup_path(target: &Path) -> Result<PathBuf> {
    let parent = target
        .parent()
        .with_context(|| format!("projection path has no parent: {}", target.display()))?;
    let file_name = target
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid projection filename {}", target.display()))?;
    for attempt in 0..100_u32 {
        let path = parent.join(format!(
            ".{file_name}.{}.backup.{attempt}.tmp",
            std::process::id()
        ));
        if !path.exists() {
            return Ok(path);
        }
    }
    bail!(
        "unable to allocate projection backup for {}",
        target.display()
    )
}

fn apply_staged<F>(staged: Vec<StagedFile>, before_replace: &mut F) -> Result<()>
where
    F: FnMut(usize, &Path) -> io::Result<()>,
{
    let mut applied = Vec::with_capacity(staged.len());
    for (index, file) in staged.iter().enumerate() {
        if let Err(error) = before_replace(index, &file.target) {
            let rollback = rollback(&applied);
            cleanup_temporaries(&staged);
            return combine_failure(error.into(), rollback);
        }

        let backup = if file.target.exists() {
            let backup = match reserve_backup_path(&file.target) {
                Ok(backup) => backup,
                Err(error) => {
                    let rollback = rollback(&applied);
                    cleanup_temporaries(&staged);
                    return combine_failure(error, rollback);
                }
            };
            if let Err(error) = fs::rename(&file.target, &backup) {
                let rollback = rollback(&applied);
                cleanup_temporaries(&staged);
                return combine_failure(
                    anyhow::Error::new(error)
                        .context(format!("backing up projection {}", file.target.display())),
                    rollback,
                );
            }
            Some(backup)
        } else {
            None
        };

        if let Err(error) = fs::rename(&file.temporary, &file.target) {
            let current_restore = restore_target(&file.target, backup.as_deref());
            let prior_restore = rollback(&applied);
            cleanup_temporaries(&staged);
            let rollback = current_restore.and(prior_restore);
            return combine_failure(
                anyhow::Error::new(error)
                    .context(format!("replacing projection {}", file.target.display())),
                rollback,
            );
        }
        applied.push(AppliedFile {
            target: file.target.clone(),
            backup,
        });
    }

    for file in applied {
        if let Some(backup) = file.backup {
            fs::remove_file(&backup)
                .with_context(|| format!("removing projection backup {}", backup.display()))?;
        }
    }
    Ok(())
}

fn rollback(applied: &[AppliedFile]) -> Result<()> {
    for file in applied.iter().rev() {
        restore_target(&file.target, file.backup.as_deref())?;
    }
    Ok(())
}

fn restore_target(target: &Path, backup: Option<&Path>) -> Result<()> {
    if target.exists() {
        fs::remove_file(target)
            .with_context(|| format!("removing partial projection {}", target.display()))?;
    }
    if let Some(backup) = backup {
        fs::rename(backup, target)
            .with_context(|| format!("restoring projection {}", target.display()))?;
    }
    Ok(())
}

fn cleanup_temporaries(staged: &[StagedFile]) {
    for file in staged {
        let _ = fs::remove_file(&file.temporary);
    }
}

fn combine_failure(error: anyhow::Error, rollback: Result<()>) -> Result<()> {
    match rollback {
        Ok(()) => Err(error),
        Err(rollback_error) => {
            Err(error.context(format!("rollback also failed: {rollback_error:#}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use tempfile::TempDir;

    #[test]
    fn failed_batch_restores_all_previous_outputs() {
        let temp = TempDir::new().unwrap();
        std::fs::write(temp.path().join("first.md"), "old first").unwrap();
        std::fs::write(temp.path().join("second.md"), "old second").unwrap();
        let files = [
            ProjectionFile::new("first.md", "new first"),
            ProjectionFile::new("second.md", "new second"),
        ];

        let error = write_projection_with_hook(&files, temp.path(), false, |index, _| {
            if index == 1 {
                Err(io::Error::other("injected commit failure"))
            } else {
                Ok(())
            }
        })
        .unwrap_err();

        assert!(error.to_string().contains("injected commit failure"));
        assert_eq!(
            std::fs::read_to_string(temp.path().join("first.md")).unwrap(),
            "old first"
        );
        assert_eq!(
            std::fs::read_to_string(temp.path().join("second.md")).unwrap(),
            "old second"
        );
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 2);
    }
}
