const TAG_PALETTE = [
  "var(--tag-blue)",
  "var(--tag-green)",
  "var(--tag-orange)",
  "var(--tag-purple)",
  "var(--tag-pink)",
  "var(--tag-cyan)",
  "var(--tag-yellow)",
  "var(--tag-red)",
] as const;

/** Return a stable palette color for a tag without storing presentation data. */
export function tagColor(tag: string): string {
  let hash = 2166136261;
  for (const character of tag.trim().toLocaleLowerCase()) {
    hash ^= character.codePointAt(0) ?? 0;
    hash = Math.imul(hash, 16777619);
  }
  // Mix the high bits before taking the small palette modulus; FNV's low
  // bits alone cluster similarly shaped names into too few buckets.
  hash ^= hash >>> 16;
  hash = Math.imul(hash, 0x85ebca6b);
  hash ^= hash >>> 13;
  return TAG_PALETTE[(hash >>> 0) % TAG_PALETTE.length];
}
