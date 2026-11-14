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
  const authority = value.slice(value.indexOf("://") + 3).split("/")[0];
  if (authority.endsWith(":") || authority.includes("%")
      || Array.from(authority).some(ch => ch.charCodeAt(0) > 127)) return false;
  try {
    const url = new URL(value);
    if (!url.hostname.startsWith("[")) {
      const validLabel = (label: string) => label.length > 0
        && !label.startsWith("-") && !label.endsWith("-")
        && Array.from(label).every(ch => {
          const code = ch.charCodeAt(0);
          return (code >= 48 && code <= 57)
            || (code >= 65 && code <= 90)
            || (code >= 97 && code <= 122)
            || code === 45;
        });
      if (!url.hostname.split(".").every(validLabel)) return false;
    }
    return (url.protocol === "https:" || url.protocol === "http:")
      && url.hostname.length > 0
      && url.username.length === 0
      && url.password.length === 0
      && url.port !== "0"
      && !url.search && !url.hash;
  } catch {
    return false;
  }
}

export function configFromEnv(
  env: Record<string, string | undefined> = process.env,
): ClientConfig {
  const baseUrl = env["DECLMIG_API_BASE"];
  if (!baseUrl || !isValidBaseUrl(baseUrl)) {
    throw new ClientError("invalid_base");
  }
  return {
    baseUrl,
    bearerToken: env["DECLMIG_TOKEN"] || undefined,
    maxResponseBytes: 64 * 1024,
  };
}

