# lib-core validation consumer

These client request-boundary adapters import the public validation packages from `declmig-lib-core`; they do not copy schemas or import server packages.

Requests validate `RequestMeta` before transport. Problem responses validate `ProblemDetails` after transport. Route-specific payload validators are selected by `ORESoftware/api-docs` operation IDs once reviewed bindings exist in `declmig-interfaces`.
