//! Deterministic seed generation for reproducible property-based tests.

/// Return a stable `u64` seed derived from a test name via FNV-1a.
///
/// Same input always produces the same output; different names produce
/// different seeds. Useful for seeding RNGs in property tests.
pub fn deterministic_seed(test_name: &str) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    test_name.bytes().fold(FNV_OFFSET, |acc, b| {
        acc.wrapping_mul(FNV_PRIME) ^ (b as u64)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_name_same_seed() {
        assert_eq!(deterministic_seed("alpha"), deterministic_seed("alpha"));
    }

    #[test]
    fn different_names_different_seeds() {
        assert_ne!(deterministic_seed("test_a"), deterministic_seed("test_b"));
    }

    #[test]
    fn empty_string_is_stable() {
        assert_eq!(deterministic_seed(""), deterministic_seed(""));
    }
}
