import * as entriesBridge from "$lib/bridge/entries";
import type { EntryHealthReport, HealthIssue } from "$lib/bridge/types";

const EMPTY_REPORT: EntryHealthReport = {
  weak: [],
  reused: [],
};

let report = $state<EntryHealthReport>(EMPTY_REPORT);
let loading = $state(false);
let requestId = 0;

export const health = {
  get report() {
    return report;
  },
  get loading() {
    return loading;
  },
  ids(issue: HealthIssue) {
    return report[issue];
  },
  async refresh() {
    const currentRequest = ++requestId;
    loading = true;
    try {
      const next = await entriesBridge.entriesHealthReport();
      if (currentRequest === requestId) report = next;
    } catch (error) {
      if (currentRequest === requestId) {
        console.error("Failed to load vault health", error);
      }
    } finally {
      if (currentRequest === requestId) loading = false;
    }
  },
  reset() {
    requestId += 1;
    report = EMPTY_REPORT;
    loading = false;
  },
};
