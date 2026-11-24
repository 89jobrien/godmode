//! git-status — SessionStart hook.
//! Prints the branch, upstream divergence, change counts and the first changed
//! paths, so a fresh session starts knowing the state of the working tree.

use std::path::Path;

use crate::config::{Config, GitStatus};
use crate::integrations::subprocess::command_in;

#[derive(Debug, Default, PartialEq, Eq)]
struct Snapshot {
    head: Option<String>,
    oid: Option<String>,
    upstream: Option<String>,
    ahead_behind: Option<(u32, u32)>,
    staged: usize,
    unstaged: usize,
    untracked: usize,
    conflicted: usize,
    changes: Vec<String>,
}

/// Run the git-status hook. Returns text for stdout (empty outside a git repo).
pub fn run(root: &Path) -> String {
    // --no-optional-locks: a read-only status must not contend with a concurrent
    // writer for the index lock.
    let Some(porcelain) = git_output(
        root,
        &[
            "--no-optional-locks",
            "status",
            "--porcelain=v2",
            "--branch",
        ],
    ) else {
        return String::new();
    };
    let stashes = git_output(root, &["--no-optional-locks", "stash", "list"])
        .map_or(0, |out| out.lines().count());
    let cfg = Config::load(root).git_status;
    render(&parse(&porcelain), stashes, &cfg)
}

fn git_output(root: &Path, args: &[&str]) -> Option<String> {
    let output = command_in("git", args, root).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn parse(porcelain: &str) -> Snapshot {
    let mut snap = Snapshot::default();
    for line in porcelain.lines() {
        if let Some(rest) = line.strip_prefix("# branch.oid ") {
            snap.oid = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.head ") {
            snap.head = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.upstream ") {
            snap.upstream = Some(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            snap.ahead_behind = parse_ahead_behind(rest);
        } else if let Some(rest) = line.strip_prefix("1 ") {
            record_tracked(&mut snap, rest, 8);
        } else if let Some(rest) = line.strip_prefix("2 ") {
            record_tracked(&mut snap, rest, 9);
        } else if let Some(rest) = line.strip_prefix("u ") {
            let mut fields = rest.splitn(10, ' ');
            let xy = fields.next().unwrap_or("UU");
            if let Some(path) = fields.nth(8) {
                snap.conflicted += 1;
                snap.changes.push(format!("{} {path}", display_xy(xy)));
            }
        } else if let Some(path) = line.strip_prefix("? ") {
            snap.untracked += 1;
            snap.changes.push(format!("?? {path}"));
        }
    }
    snap
}

/// Record an ordinary (`1`) or renamed/copied (`2`) entry. `fields` is the
/// number of space-separated fields after the entry marker, the last being the path.
fn record_tracked(snap: &mut Snapshot, rest: &str, fields: usize) {
    let mut parts = rest.splitn(fields, ' ');
    let xy = parts.next().unwrap_or("..");
    let Some(path) = parts.last() else { return };
    let mut chars = xy.chars();
    if chars.next().is_some_and(|x| x != '.') {
        snap.staged += 1;
    }
    if chars.next().is_some_and(|y| y != '.') {
        snap.unstaged += 1;
    }
    let path = match path.split_once('\t') {
        Some((new, old)) => format!("{new} (from {old})"),
        None => path.to_string(),
    };
    snap.changes.push(format!("{} {path}", display_xy(xy)));
}

fn display_xy(xy: &str) -> String {
    xy.chars().map(|c| if c == '.' { ' ' } else { c }).collect()
}

fn parse_ahead_behind(raw: &str) -> Option<(u32, u32)> {
    let (ahead, behind) = raw.split_once(' ')?;
    Some((
        ahead.strip_prefix('+')?.parse().ok()?,
        behind.strip_prefix('-')?.parse().ok()?,
    ))
}

fn render(snap: &Snapshot, stashes: usize, cfg: &GitStatus) -> String {
    let head = snap.head.as_deref().unwrap_or("unknown");
    let branch = if head == "(detached)" {
        let short = snap.oid.as_deref().unwrap_or("unknown");
        format!("(detached at {})", &short[..short.len().min(7)])
    } else {
        head.to_string()
    };

    let mut out = format!("[git-status] branch: {branch}");
    match (snap.upstream.as_deref(), snap.ahead_behind) {
        (Some(up), Some((ahead, behind))) => {
            out.push_str(&format!("  upstream: {up}  ahead {ahead} behind {behind}"));
        }
        (Some(up), None) => out.push_str(&format!("  upstream: {up}  (gone)")),
        (None, _) => out.push_str("  upstream: none"),
    }
    out.push('\n');

    if cfg.protected_branches.iter().any(|b| b == head) {
        out.push_str(&format!(
            "[git-status] WARNING: on protected branch '{head}'\n"
        ));
    }

    let stash_note = if stashes > 0 {
        format!("  stashes {stashes}")
    } else {
        String::new()
    };
    if snap.changes.is_empty() {
        out.push_str(&format!("[git-status] working tree clean{stash_note}\n"));
        return out;
    }

    out.push_str(&format!(
        "[git-status] staged {}  unstaged {}  untracked {}  conflicted {}{stash_note}\n",
        snap.staged, snap.unstaged, snap.untracked, snap.conflicted
    ));
    for change in snap.changes.iter().take(cfg.max_paths) {
        out.push_str(&format!("  {change}\n"));
    }
    let hidden = snap.changes.len().saturating_sub(cfg.max_paths);
    if hidden > 0 {
        out.push_str(&format!("  ... and {hidden} more\n"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const MIXED: &str = "# branch.oid abcdef1234567890\n# branch.head develop\n# branch.upstream origin/develop\n# branch.ab +3 -1\n1 .M N... 100644 100644 100644 aaaa bbbb Cargo.lock\n1 M. N... 100644 100644 100644 aaaa bbbb src/staged.rs\n1 MM N... 100644 100644 100644 aaaa bbbb src/both.rs\n2 R. N... 100644 100644 100644 aaaa bbbb R100 src/new.rs\tsrc/old.rs\nu UU N... 100644 100644 100644 100644 aaaa bbbb cccc src/conflict.rs\n? notes.txt\n";

    fn cfg(max_paths: usize, protected: &[&str]) -> GitStatus {
        GitStatus {
            max_paths,
            protected_branches: protected.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn parses_branch_and_divergence() {
        let snap = parse(MIXED);
        assert_eq!(snap.head.as_deref(), Some("develop"));
        assert_eq!(snap.upstream.as_deref(), Some("origin/develop"));
        assert_eq!(snap.ahead_behind, Some((3, 1)));
    }

    #[test]
    fn counts_each_kind_of_change() {
        let snap = parse(MIXED);
        // staged: src/staged.rs, src/both.rs, the rename
        assert_eq!(snap.staged, 3);
        // unstaged: Cargo.lock, src/both.rs
        assert_eq!(snap.unstaged, 2);
        assert_eq!(snap.untracked, 1);
        assert_eq!(snap.conflicted, 1);
        assert_eq!(snap.changes.len(), 6);
    }

    #[test]
    fn renders_paths_with_short_status_codes() {
        let out = render(&parse(MIXED), 0, &cfg(30, &[]));
        assert!(out.contains("  .M Cargo.lock") || out.contains("   M Cargo.lock"));
        assert!(out.contains("  ?? notes.txt"));
        assert!(out.contains("src/new.rs (from src/old.rs)"));
        assert!(out.contains("UU src/conflict.rs"));
    }

    #[test]
    fn header_line_reports_upstream_divergence() {
        let out = render(&parse(MIXED), 0, &cfg(30, &[]));
        assert!(out.starts_with(
            "[git-status] branch: develop  upstream: origin/develop  ahead 3 behind 1\n"
        ));
    }

    #[test]
    fn clean_tree_says_so_and_notes_stashes() {
        let clean = "# branch.oid abc\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +0 -0\n";
        let out = render(&parse(clean), 2, &cfg(30, &[]));
        assert!(out.contains("working tree clean  stashes 2"));
        assert!(!out.contains("staged"));
    }

    #[test]
    fn caps_listed_paths_and_reports_the_rest() {
        let out = render(&parse(MIXED), 0, &cfg(2, &[]));
        assert_eq!(out.matches("\n  ").count(), 3, "2 paths + overflow: {out}");
        assert!(out.contains("... and 4 more"));
    }

    #[test]
    fn detached_head_shows_short_oid() {
        let detached = "# branch.oid 0123456789abcdef\n# branch.head (detached)\n";
        let out = render(&parse(detached), 0, &cfg(30, &[]));
        assert!(out.contains("branch: (detached at 0123456)"));
        assert!(out.contains("upstream: none"));
    }

    #[test]
    fn upstream_without_ahead_behind_is_gone() {
        let gone = "# branch.oid abc\n# branch.head topic\n# branch.upstream origin/topic\n";
        let out = render(&parse(gone), 0, &cfg(30, &[]));
        assert!(out.contains("upstream: origin/topic  (gone)"));
    }

    #[test]
    fn protected_branch_gets_a_warning() {
        let out = render(&parse(MIXED), 0, &cfg(30, &["develop"]));
        assert!(out.contains("WARNING: on protected branch 'develop'"));
        let quiet = render(&parse(MIXED), 0, &cfg(30, &["main"]));
        assert!(!quiet.contains("WARNING"));
    }

    #[test]
    fn run_reports_untracked_file_in_a_real_repo() {
        use crate::git_harness::GitRepo;

        let repo = GitRepo::init();
        repo.commit("initial", Some("2020-01-01T00:00:00+00:00"));
        std::fs::write(repo.path().join("scratch.txt"), "x").expect("write file");
        let out = run(repo.path());
        assert!(out.starts_with("[git-status] branch: "), "{out}");
        assert!(out.contains("untracked 1"), "{out}");
        assert!(out.contains("?? scratch.txt"), "{out}");
    }

    #[test]
    fn run_is_silent_outside_a_git_repo() {
        let tmp = tempfile::tempdir().expect("tempdir");
        assert_eq!(run(tmp.path()), "");
    }
}
