import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import * as totpBridge from "$lib/bridge/totp";
import { clipboard } from "$lib/stores/clipboard.svelte";
import { toast } from "$lib/stores/toast.svelte";
import TOTPField from "./TOTPField.svelte";

vi.mock("$lib/bridge/totp", () => ({ totpCompute: vi.fn() }));
vi.mock("$lib/stores/clipboard.svelte", () => ({
  clipboard: { copy: vi.fn() },
}));

afterEach(() => {
  for (const item of toast.all) toast.dismiss(item.id);
  vi.restoreAllMocks();
});

describe("TOTPField", () => {
  it("announces a successful copy", async () => {
    vi.mocked(totpBridge.totpCompute).mockResolvedValue({
      code: "123456",
      remaining: 20,
      period: 30,
    });
    vi.mocked(clipboard.copy).mockResolvedValue(undefined);
    render(TOTPField, { entryId: "entry-1" });
    await waitFor(() => expect(screen.getByText("123 456")).toBeInTheDocument());

    await fireEvent.click(screen.getByRole("button", { name: "Copy TOTP code" }));

    expect(clipboard.copy).toHaveBeenCalledWith("123456");
    expect(toast.all.at(-1)).toMatchObject({ kind: "success", message: "TOTP code copied" });
  });
});
