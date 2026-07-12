import { beforeEach, describe, expect, it } from "vitest";
import { saveStatus } from "./saveStatus.svelte";

beforeEach(() => saveStatus.markSaved());

describe("save status", () => {
  it("tracks dirty, saving, error, and saved transitions", () => {
    saveStatus.markDirty();
    expect(saveStatus.state).toBe("dirty");

    saveStatus.markSaving();
    expect(saveStatus.state).toBe("saving");

    saveStatus.markError("Disk changed");
    expect(saveStatus.state).toBe("error");
    expect(saveStatus.errorMessage).toBe("Disk changed");

    saveStatus.markSaved();
    expect(saveStatus.state).toBe("saved");
    expect(saveStatus.errorMessage).toBe("");
  });
});
