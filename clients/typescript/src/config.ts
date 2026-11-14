import { ClientError } from "./errors";

export interface ClientConfig {
  baseUrl: string;
  bearerToken?: string;
  maxResponseBytes: number;
}

/** Reject credentials, ambiguous URLs and non-HTTP transports at the SDK boundary. */
export function isValidBaseUrl(value: string): boolean {
  // URL() rejects invalid ports/IPv6 syntax; inspect the original text too,
  // because URL normalization can silently encode spaces and backslashes.
  if (Array.from(value).some(ch =>
    ch.trim().length === 0 || ch.charCodeAt(0) === 92 || ch === "?" || ch === "#"
  )) return false;
  try {
    const url = new URL(value);
    return (url.protocol === "https:" || url.protocol === "http:")
      && url.hostname.length > 0
      && url.username.length === 0
      && url.password.length === 0
      && !url.search && !url.hash;
  } catch {
    return false;
  }
}

export function configFromEnv(
  env: Record<string, string | undefined> = process.env,
): ClientConfig {
  const baseUrl = env["DECLMIG_API_BASE"]?.trim();
  if (!baseUrl || !isValidBaseUrl(baseUrl)) {
    throw new ClientError("invalid_base");
  }
  return {
    baseUrl,
    bearerToken: env["DECLMIG_TOKEN"] || undefined,
    maxResponseBytes: 64 * 1024,
  };
}

