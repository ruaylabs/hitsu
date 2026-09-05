export interface TagColors {
  readonly fill: string;
  readonly text: string;
}

const TAG_PALETTE = [
  { fill: "var(--tag-blue-fill)", text: "var(--tag-blue-text)" },
  { fill: "var(--tag-green-fill)", text: "var(--tag-green-text)" },
  { fill: "var(--tag-orange-fill)", text: "var(--tag-orange-text)" },
  { fill: "var(--tag-purple-fill)", text: "var(--tag-purple-text)" },
  { fill: "var(--tag-pink-fill)", text: "var(--tag-pink-text)" },
  { fill: "var(--tag-cyan-fill)", text: "var(--tag-cyan-text)" },
  { fill: "var(--tag-yellow-fill)", text: "var(--tag-yellow-text)" },
  { fill: "var(--tag-red-fill)", text: "var(--tag-red-text)" },
] as const satisfies readonly TagColors[];

/** Return stable fill and text colors for a tag without storing presentation data. */
export function tagColor(tag: string): TagColors {
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
