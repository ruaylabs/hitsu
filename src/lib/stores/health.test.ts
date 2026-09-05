import { beforeEach, describe, expect, it, vi } from "vitest";
import * as entriesBridge from "$lib/bridge/entries";
import { health } from "./health.svelte";

vi.mock("$lib/bridge/entries", () => ({
  entriesHealthReport: vi.fn(),
}));

const entriesHealthReportMock = vi.mocked(entriesBridge.entriesHealthReport);

describe("health store", () => {
  beforeEach(() => {
    health.reset();
    entriesHealthReportMock.mockReset();
  });

  it("loads issue IDs without receiving password values", async () => {
    entriesHealthReportMock.mockResolvedValue({
      weak: ["weak-id"],
      reused: ["reused-id"],
    });

    await health.refresh();

    expect(health.ids("weak")).toEqual(["weak-id"]);
    expect(health.ids("reused")).toEqual(["reused-id"]);
    expect(health.loading).toBe(false);
  });

  it("clears report metadata when the vault locks", async () => {
    entriesHealthReportMock.mockResolvedValue({
      weak: ["weak-id"],
      reused: [],
    });
    await health.refresh();

    health.reset();

    expect(health.report.weak).toEqual([]);
  });
});
