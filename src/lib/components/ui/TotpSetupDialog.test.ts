import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import jsQR from "jsqr";
import { beforeEach, describe, expect, it, vi } from "vitest";
import TotpSetupDialog from "./TotpSetupDialog.svelte";

vi.mock("jsqr", () => ({ default: vi.fn() }));

const mockedJsQr = vi.mocked(jsQR);

beforeEach(() => {
  mockedJsQr.mockReset();
  vi.stubGlobal(
    "createImageBitmap",
    vi.fn().mockResolvedValue({ width: 320, height: 320, close: vi.fn() }),
  );
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue({
    drawImage: vi.fn(),
    getImageData: vi.fn().mockReturnValue({
      data: new Uint8ClampedArray(4),
      width: 1,
      height: 1,
    }),
  } as unknown as CanvasRenderingContext2D);
});

describe("TotpSetupDialog", () => {
  it("imports a QR seed using the same defaults as manual entry", async () => {
    mockedJsQr.mockReturnValue({
      data: "otpauth://totp/Example:alice?secret=jbswy3dpehpk3pxp&issuer=Example&digits=8",
    } as ReturnType<typeof jsQR>);
    const onconfirm = vi.fn();
    const { container } = render(TotpSetupDialog, { onconfirm, oncancel: vi.fn() });
    const input = container.querySelector<HTMLInputElement>("#totp-qr-file");
    if (!input) throw new Error("QR file input was not rendered");

    await fireEvent.change(input, {
      target: { files: [new File(["qr"], "totp.png", { type: "image/png" })] },
    });
    await waitFor(() => expect(screen.getByText("QR code ready")).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onconfirm).toHaveBeenCalledWith(
      "otpauth://totp/entry?secret=JBSWY3DPEHPK3PXP&period=30&digits=6",
    );
  });

  it("rejects QR codes that do not contain a TOTP setup link", async () => {
    mockedJsQr.mockReturnValue({ data: "https://example.com" } as ReturnType<typeof jsQR>);
    const { container } = render(TotpSetupDialog, {
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    const input = container.querySelector<HTMLInputElement>("#totp-qr-file");
    if (!input) throw new Error("QR file input was not rendered");

    await fireEvent.change(input, {
      target: { files: [new File(["qr"], "not-totp.png", { type: "image/png" })] },
    });

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The QR code is not a TOTP setup code",
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("still supports entering a seed code manually", async () => {
    const onconfirm = vi.fn();
    render(TotpSetupDialog, { onconfirm, oncancel: vi.fn() });

    await fireEvent.click(screen.getByRole("tab", { name: "Seed code" }));
    await fireEvent.input(screen.getByLabelText("Seed code"), {
      target: { value: "jbsw-y3dp ehpk3pxp" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onconfirm).toHaveBeenCalledWith(
      "otpauth://totp/entry?secret=JBSWY3DPEHPK3PXP&period=30&digits=6",
    );
  });
});
