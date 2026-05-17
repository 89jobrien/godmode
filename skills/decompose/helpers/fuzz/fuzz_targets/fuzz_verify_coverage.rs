#![no_main]
use libfuzzer_sys::fuzz_target;

#[derive(Debug)]
struct CoverageResult {
    ok: bool,
    orphaned: Vec<String>,
    duplicated: Vec<String>,
}

fn verify_coverage(source: &[String], splits: &[Vec<String>]) -> CoverageResult {
    let mut covered: Vec<&str> = Vec::new();
    for split in splits {
        for f in split {
            covered.push(f.as_str());
        }
    }

    let orphaned: Vec<String> = source
        .iter()
        .filter(|f| !covered.contains(&f.as_str()))
        .cloned()
        .collect();

    let mut seen = std::collections::HashSet::new();
    let duplicated: Vec<String> = covered
        .iter()
        .filter(|f| !seen.insert(*f))
        .map(|s| s.to_string())
        .collect();

    CoverageResult {
        ok: orphaned.is_empty() && duplicated.is_empty(),
        orphaned,
        duplicated,
    }
}

fuzz_target!(|data: &[u8]| {
    // Interpret the fuzz input as a newline-delimited list of path strings.
    // First line = number of "source" files (parsed as usize, clamped).
    // Remaining lines alternate: file path tokens split by '|' into splits.
    //
    // We just need verify_coverage to not panic on any input shape.
    if let Ok(s) = std::str::from_utf8(data) {
        let lines: Vec<&str> = s.lines().collect();
        if lines.is_empty() {
            return;
        }

        // Build source list from all unique lines
        let source: Vec<String> = lines
            .iter()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect();

        if source.is_empty() {
            return;
        }

        // Build two arbitrary splits: first half and second half of source
        let mid = source.len() / 2;
        let split_a = source[..mid].to_vec();
        let split_b = source[mid..].to_vec();
        let splits = vec![split_a, split_b];

        let result = verify_coverage(&source, &splits);

        // Invariants that must always hold regardless of input:
        // 1. ok == (orphaned.is_empty() && duplicated.is_empty())
        assert_eq!(
            result.ok,
            result.orphaned.is_empty() && result.duplicated.is_empty(),
            "ok flag inconsistent with orphaned/duplicated lists"
        );

        // 2. A clean partition of source into two halves has no orphans and no dups.
        //    (This only holds when source has no duplicate entries itself.)
        let unique_source: std::collections::HashSet<&str> =
            source.iter().map(|s| s.as_str()).collect();
        if unique_source.len() == source.len() {
            // Source has no duplicates — our partition must be clean
            assert!(
                result.orphaned.is_empty(),
                "no orphans expected for a clean partition, got: {:?}",
                result.orphaned
            );
            assert!(
                result.duplicated.is_empty(),
                "no dups expected for a clean partition, got: {:?}",
                result.duplicated
            );
            assert!(result.ok, "ok must be true for a clean partition");
        }

        // 3. Overlapping splits always produce duplicates.
        let overlapping_splits = vec![source.clone(), source.clone()];
        let overlap_result = verify_coverage(&source, &overlapping_splits);
        if source.len() > 1 {
            assert!(
                !overlap_result.ok,
                "overlapping splits must not be ok for non-trivial source"
            );
        }
    }
});
