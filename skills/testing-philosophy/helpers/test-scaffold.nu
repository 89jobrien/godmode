#!/usr/bin/env nu
# test-scaffold.nu — thin shim; logic lives in godmode-core/src/scaffold.rs
# Usage: nu skills/testing-philosophy/helpers/test-scaffold.nu <crate> <dimension>

def main [crate: string, dimension: string] {
    run-external "godmode" "scaffold" $crate $dimension
}
