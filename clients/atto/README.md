# AttoLang client candidate (not release-admitted)

`src/client.atto` is an AttoLang **library source** with a typed health DTO,
a migration-plan header and pure validation helpers matching the existing
`/v1/health` SDK boundary: a 64 KiB response budget, HTTP status 200,
nonempty bearer tokens, and nonempty plan IDs/revisions.

It does **not** implement HTTP requests, JSON parsing, bearer-header emission,
authentication, or the opaque `payload` field. Those must be supplied by an
independently reviewed transport/decoder and admitted contract IR before the
Atto client becomes a complete Tier 1 SDK. Do not infer production readiness
or TypeSpec/JSON Schema parity from successful syntax compilation.

To compile the smoke fixture with a pinned `attoc`, concatenate the library
source followed by `tests/smoke.atto`, then compile the combined UTF-8 file.
The smoke fixture returns 0 when the checked policy cases hold. CI currently
**compiles** the fixture but does not execute the guest Wasm; runtime behavior
must be verified separately on every intended backend. Rust counterpart tests
run under `cargo test --manifest-path clients/rust/Cargo.toml`.

The source of truth is `declarative-migrations/declmig-interfaces`. Its
independent TypeSpec and JSON Schema/OpenAPI authorities are unchanged.
