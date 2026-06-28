// RFC 4648 Base32 decoding
//
// TOTP storage is KeepassXC-compatible: the Base32 secret is stored
// as a protected field `TOTP Seed` and the period/digits as an
// unprotected field `TOTP Settings` (e.g. `30;6`).
const base32Chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

function base32Decode(s: string): Uint8Array {
  const clean = s.replace(/[^A-Za-z2-7]/g, "").toUpperCase();
  const bytes: number[] = [];
  let buffer = 0;
  let bitsLeft = 0;

  for (const ch of clean) {
    const val = base32Chars.indexOf(ch);
    if (val === -1) continue;
    buffer = (buffer << 5) | val;
    bitsLeft += 5;
    if (bitsLeft >= 8) {
      bitsLeft -= 8;
      bytes.push((buffer >> bitsLeft) & 0xff);
    }
  }
  return new Uint8Array(bytes);
}

function intToBytesBE(num: bigint, length: number): Uint8Array {
  const arr = new Uint8Array(length);
  for (let i = length - 1; i >= 0; i--) {
    arr[i] = Number(num & BigInt(0xff));
    num >>= BigInt(8);
  }
  return arr;
}

async function hmacSha1(key: Uint8Array, data: Uint8Array): Promise<Uint8Array> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key,
    { name: "HMAC", hash: "SHA-1" },
    false,
    ["sign"],
  );
  const sig = await crypto.subtle.sign("HMAC", cryptoKey, data);
  return new Uint8Array(sig);
}

function truncate(hs: Uint8Array, digits: number): number {
  const offset = hs[19] & 0xf;
  const binCode =
    ((hs[offset] & 0x7f) << 24) |
    ((hs[offset + 1] & 0xff) << 16) |
    ((hs[offset + 2] & 0xff) << 8) |
    (hs[offset + 3] & 0xff);
  return binCode % 10 ** digits;
}

export interface TotpParams {
  secret: Uint8Array;
  digits: number;
  period: number;
  algorithm: string;
  issuer?: string;
  label?: string;
}

export function parseOtpauthUri(uri: string): TotpParams | null {
  try {
    const u = new URL(uri);
    if (u.protocol !== "otpauth:") return null;

    const label = decodeURIComponent(u.pathname.replace(/^\//, ""));
    const secret = u.searchParams.get("secret");
    if (!secret) return null;

    return {
      secret: base32Decode(secret),
      digits: parseInt(u.searchParams.get("digits") || "6", 10),
      period: parseInt(u.searchParams.get("period") || "30", 10),
      algorithm: u.searchParams.get("algorithm") || "SHA1",
      issuer: u.searchParams.get("issuer") || undefined,
      label: label || undefined,
    };
  } catch {
    return null;
  }
}

export async function computeTotp(params: TotpParams): Promise<string> {
  const now = Math.floor(Date.now() / 1000);
  const counter = Math.floor(now / params.period);
  const counterBytes = intToBytesBE(BigInt(counter), 8);
  const hs = await hmacSha1(params.secret, counterBytes);
  const code = truncate(hs, params.digits);
  return code.toString().padStart(params.digits, "0");
}

/** Returns seconds remaining in the current TOTP period */
export function totpRemainingSeconds(period: number): number {
  return period - (Math.floor(Date.now() / 1000) % period);
}
