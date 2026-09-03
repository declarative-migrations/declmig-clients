import assert from "node:assert/strict";
import test from "node:test";
import { validatedRequestHeaders } from "../src/index.js";

test("validates request metadata before transport", () => {
  assert.deepEqual(validatedRequestHeaders({requestId: "req-1", traceId: "trace-1"}), {"x-request-id": "req-1", traceparent: "trace-1"});
});

test("rejects client supplied server identity", () => {
  assert.throws(() => validatedRequestHeaders({requestId: "req-1", traceId: "trace-1", userId: "not-public"}));
});
