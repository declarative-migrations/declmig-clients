import { ClientError } from "./errors";

export interface ClientConfig {
  baseUrl: string;
  bearerToken?: string;
  maxResponseBytes: number;
}

/** Reject credentials, ambiguous URLs and non-HTTP transports at the SDK boundary. */
export function isValidBaseUrl(value: string): boolean {
  const rest = value.startsWith("https://") ? value.slice(8)
    : value.startsWith("http://") ? value.slice(7) : "";
  const authority = rest.split("/")[0];
  return authority.length > 0 && !authority.includes("@")
    && !/[\\s?#\\\\]/.test(rest);
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

