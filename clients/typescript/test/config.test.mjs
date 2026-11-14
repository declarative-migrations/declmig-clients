import { test } from "node:test";
import assert from "node:assert/strict";
import { isValidBaseUrl, configFromEnv } from "../dist/config.mjs";

const valid = [
  "http://127.0.0.1:8080",
  "http://localhost:8080",
  "https://api.example.test/v1",
  "https://[::1]:8443/api",
];
const invalid = [
  "", "ftp://example.test", "https://",
  "https://u:p@example.test", "https://example.test?q=1",
  "https://example.test/#x", "https://example.test/white space",
  "https://:123", "https://api.example.test:bad",
  "https://api.example.test:0", "https://api.example.test:",
  "https://999.999.999.999", "https://bad_name.example",
  "https://[::1", "https://api.example.test\\\\path",
  "https://api.example.test%40evil", " https://api.example.test ",
];

test("TypeScript API base URL validator accepts only expected HTTP(S) targets", () => {
  for (const value of valid) {
    assert.ok(isValidBaseUrl(value), "unexpectedly rejected: " + value);
  }
  for (const value of invalid) {
    assert.equal(isValidBaseUrl(value), false, "unexpectedly accepted: " + value);
  }
});

test("invalid endpoint is rejected without printing credentials", () => {
  for (const value of invalid) {
    assert.throws(
      () => configFromEnv({ DECLMIG_API_BASE: value, DECLMIG_TOKEN: "topsecret" }),
      error => error instanceof Error && !error.message.includes("topsecret"),
      "value should be rejected without credential leakage"
    );
  }
});

test("configured endpoint remains unmodified and token is not copied into URL", () => {
  const config = configFromEnv({
    DECLMIG_API_BASE: "https://api.example.test/v1",
    DECLMIG_TOKEN: "topsecret",
  });
  assert.equal(config.baseUrl, "https://api.example.test/v1");
  assert.equal(config.bearerToken, "topsecret");
  assert.ok(!config.baseUrl.includes(config.bearerToken));
});
