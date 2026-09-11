import { ProblemDetailsSchema, RequestMetaSchema, type ProblemDetails, type RequestMeta } from "@declarative-migrations/declmig-validation";

export function validatedRequestHeaders(value: unknown): Readonly<Record<string, string>> {
  const meta: RequestMeta = RequestMetaSchema.parse(value);
  return Object.freeze({
    "x-request-id": meta.requestId,
    traceparent: meta.traceId,
    ...(meta.locale === undefined ? {} : {"accept-language": meta.locale}),
  });
}

export function decodeProblem(value: unknown): ProblemDetails {
  return ProblemDetailsSchema.parse(value);
}
