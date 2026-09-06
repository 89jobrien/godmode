# godmode toolkit — auto-loaded by toolkit-hook when you cd into this repo

export def "build"       [] { cargo build --workspace }
export def "build-release" [] { cargo build --release -p godmode-cli }
export def "test"        [] { cargo nextest run --workspace }
export def "lint"        [] { cargo clippy --workspace -- -D warnings }
export def "fmt"         [] { cargo fmt --all }
export def "ci"          [] { just ci }
export def "conformance" [] { just conformance }
export def "install"     [] { just install }
export def "clean"       [] { cargo clean }

export def "help" [] {
    scope commands
    | where name =~ "^tk "
    | select name
    | sort-by name
}
