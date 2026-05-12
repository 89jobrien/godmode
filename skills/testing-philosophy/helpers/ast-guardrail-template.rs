// Template: AST guardrail test
//
// Parse a source file with `syn` and assert structural invariants.
// Copy this into `tests/conformance/source_guardrails.rs` and adapt
// the invariant closure to your crate's boundary rules.
//
// Requires: syn = { version = "2", features = ["full", "visit"] }

#[test]
fn no_leaked_types_across_boundary() {
    let source = std::fs::read_to_string("src/context.rs")
        .expect("failed to read source file");
    let file = syn::parse_file(&source)
        .expect("failed to parse source file");

    let mut violations = Vec::new();

    for item in &file.items {
        // Example: assert context.rs only re-exports from common crate,
        // never defines its own types.
        if let syn::Item::Struct(s) = item {
            violations.push(format!(
                "context.rs defines struct `{}` — should only re-export",
                s.ident
            ));
        }
        if let syn::Item::Enum(e) = item {
            violations.push(format!(
                "context.rs defines enum `{}` — should only re-export",
                e.ident
            ));
        }
    }

    if !violations.is_empty() {
        panic!(
            "Source guardrail violations in context.rs:\n  {}",
            violations.join("\n  ")
        );
    }
}
