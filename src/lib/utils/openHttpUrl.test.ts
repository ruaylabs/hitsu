import { openUrl } from "@tauri-apps/plugin-opener";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { openHttpUrl } from "./openHttpUrl";

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

const openUrlMock = vi.mocked(openUrl);

describe("openHttpUrl", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("opens HTTP(S) URLs and defaults bare hosts to HTTPS", () => {
    openHttpUrl("https://example.com/login");
    openHttpUrl("example.org");

    expect(openUrlMock).toHaveBeenNthCalledWith(1, "https://example.com/login");
    expect(openUrlMock).toHaveBeenNthCalledWith(2, "https://example.org");
  });

  it("blocks disallowed schemes and invalid URLs", () => {
    const warn = vi.spyOn(console, "warn").mockImplementation(() => {});

    openHttpUrl("javascript://alert");
    openHttpUrl("not a valid host");

    expect(openUrlMock).not.toHaveBeenCalled();
    expect(warn).toHaveBeenCalledTimes(2);
    warn.mockRestore();
  });
});
