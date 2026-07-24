import * as entriesBridge from "$lib/bridge/entries";
import type { EntrySummary } from "$lib/bridge/types";
import { entryHaystack } from "$lib/utils/search";

export interface EntrySearch {
  /** True when `entry` matches the current query (always true for an empty query). */
  matches(entry: EntrySummary): boolean;
}

/**
 * Reactive entry search against the backend's full-field index. Full-field
 * search stays in Rust so notes and other field values do not have to be
 * copied into every list summary; the summary-field haystack serves as an
 * immediate fallback while the backend result is in flight or unavailable.
 *
 * Must be called during component initialisation (uses `$effect`).
 *
 * @param query      Reactive source of the search query.
 * @param debounceMs Delay before hitting the backend; 0 queries immediately.
 * @param refreshOn  Optional extra reactive dependency; the backend query
 *                   re-runs when it changes (e.g. the vault's entry list).
 */
export function createEntrySearch(
  query: () => string,
  { debounceMs = 0, refreshOn }: { debounceMs?: number; refreshOn?: () => void } = {},
): EntrySearch {
  let matchIds = $state<string[] | null>(null);
  const matchIdSet = $derived(matchIds === null ? null : new Set(matchIds));
  let searchRequest = 0;

  $effect(() => {
    refreshOn?.();
    const q = query().trim();
    const request = ++searchRequest;
    matchIds = null;
    if (!q) return;

    const run = () => {
      void entriesBridge
        .entriesSearch(q)
        .then((ids) => {
          if (request === searchRequest && query().trim() === q) matchIds = ids;
        })
        .catch(() => {
          // Keep the summary-field fallback when backend search is unavailable.
        });
    };

    if (!debounceMs) {
      run();
      return;
    }
    const timeout = setTimeout(run, debounceMs);
    return () => {
      clearTimeout(timeout);
      if (searchRequest === request) searchRequest++;
    };
  });

  return {
    matches(entry: EntrySummary): boolean {
      const q = query().trim().toLowerCase();
      if (!q) return true;
      if (matchIdSet?.has(entry.id)) return true;
      return entryHaystack(entry).includes(q);
    },
  };
}
