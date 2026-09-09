//! Explicit mutation mode for filesystem operations.

/// Selects whether an operation previews or writes filesystem changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum WriteMode {
    /// Return planned paths without changing the filesystem.
    Preview,
    /// Apply filesystem changes.
    Write,
}

impl WriteMode {
    /// Convert a legacy dry-run flag (`true` means preview).
    pub const fn from_dry_run(dry_run: bool) -> Self {
        if dry_run { Self::Preview } else { Self::Write }
    }

    /// Returns whether filesystem writes are enabled.
    pub const fn writes(self) -> bool {
        matches!(self, Self::Write)
    }
}

impl From<bool> for WriteMode {
    fn from(dry_run: bool) -> Self {
        Self::from_dry_run(dry_run)
    }
}
