# Rust validation-import boundary

The public validation consumers are release and security boundaries, not text-search fixtures. `tools/validation-import-check` is the executable authority for proving that the TypeScript, Rust, Go, and Gleam consumers depend on and import only the public validation SDKs from `declmig-lib-core`.

The checker structurally parses `package.json`, `Cargo.toml`, `go.mod`, and `gleam.toml`; verifies exact local dependency paths and package identities; inspects executable source roots rather than README or comment text; rejects server-core and migration-executor imports; rejects symlink traversal; and emits `ores.validation-import-check/v1` evidence.

The retired Python checker used runtime-removable `assert` statements and concatenated every file beneath `validation-consumer`. It therefore passed under `python3 -O`, and ordinary execution could treat a package name in a manifest or README as proof that executable source imported it. CI retains exact pre-migration probes from the parent commit so those failure modes remain regression evidence without preserving Python as the active gate.

Run:

```sh
cargo run --manifest-path tools/validation-import-check/Cargo.toml -- \
  --root . --report target/validation-import-report.json
```

Any malformed manifest, path drift, missing executable import, forbidden server dependency, or unsafe filesystem boundary exits non-zero in debug and release builds.
