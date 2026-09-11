#!/usr/bin/env python3
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
text = "\n".join(path.read_text(errors="ignore") for path in (ROOT / "validation-consumer").rglob("*") if path.is_file())
required = ["@declarative-migrations/declmig-validation", "declmig-validation", "github.com/declarative-migrations/declmig-lib-core/validation/golang", "declmig_validation"]
for dependency in required:
    assert dependency in text, f"missing public lib-core import: {dependency}"
for forbidden in ("declmig-validation-server", "golang-server", "declmig_validation_server"):
    assert forbidden not in text, f"client imported server-only package: {forbidden}"
print("all four clients import only public lib-core validation packages")
