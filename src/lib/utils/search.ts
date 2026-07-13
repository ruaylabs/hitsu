import type { EntrySummary } from "$lib/bridge/types";

// Lowercased searchable text per summary, computed once per object and
// dropped automatically when the summary is replaced (entries are swapped
// wholesale on every vault mutation). Fields are joined with "\n" so a
// query can't accidentally match across field boundaries.
const haystacks = new WeakMap<EntrySummary, string>();

export function entryHaystack(entry: EntrySummary): string {
  let h = haystacks.get(entry);
  if (h === undefined) {
    h = [entry.title, entry.subtitle, entry.url ?? "", entry.username ?? "", ...entry.tags]
      .join("\n")
      .toLowerCase();
    haystacks.set(entry, h);
  }
  return h;
}
