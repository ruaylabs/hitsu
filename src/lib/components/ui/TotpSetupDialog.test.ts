import { fireEvent, render, screen, waitFor } from "@testing-library/svelte";
import jsQR from "jsqr";
import { beforeEach, describe, expect, it, vi } from "vitest";
import TotpSetupDialog from "./TotpSetupDialog.svelte";

vi.mock("jsqr", () => ({ default: vi.fn() }));

const mockedJsQr = vi.mocked(jsQR);

async function scanQr(container: HTMLElement, data: string) {
  mockedJsQr.mockReturnValue({ data } as ReturnType<typeof jsQR>);
  const input = container.querySelector<HTMLInputElement>("#totp-qr-file");
  if (!input) throw new Error("QR file input was not rendered");
  await fireEvent.change(input, {
    target: { files: [new File(["qr"], "totp.png", { type: "image/png" })] },
  });
}

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
  it("preserves the QR label, issuer, and non-default TOTP settings", async () => {
    const onconfirm = vi.fn();
    const { container } = render(TotpSetupDialog, { onconfirm, oncancel: vi.fn() });
    await scanQr(
      container,
      "otpauth://totp/Example:alice?secret=jbswy3dpehpk3pxp" +
        "&issuer=Example&period=60&digits=8&algorithm=SHA256",
    );

    await waitFor(() => expect(screen.getByText("QR code ready")).toBeInTheDocument());
    expect(screen.getByText("Example:alice")).toBeInTheDocument();
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onconfirm).toHaveBeenCalledWith(
      "otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP&period=60" +
        "&digits=8&issuer=Example&algorithm=SHA256",
    );
  });

  it("applies standard defaults when QR settings are omitted", async () => {
    const onconfirm = vi.fn();
    const { container } = render(TotpSetupDialog, { onconfirm, oncancel: vi.fn() });
    await scanQr(container, "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP");

    await waitFor(() => expect(screen.getByText("QR code ready")).toBeInTheDocument());
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(onconfirm).toHaveBeenCalledWith(
      "otpauth://totp/alice?secret=JBSWY3DPEHPK3PXP&period=30&digits=6",
    );
  });

  it("rejects QR codes that do not contain a TOTP setup link", async () => {
    const { container } = render(TotpSetupDialog, {
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    await scanQr(container, "https://example.com");

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The QR code is not a TOTP setup code",
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("rejects unsupported Steam encoders instead of saving incorrect codes", async () => {
    const { container } = render(TotpSetupDialog, {
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    await scanQr(
      container,
      "otpauth://totp/Steam:alice?secret=JBSWY3DPEHPK3PXP&digits=5&encoder=steam",
    );

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The QR code uses an unsupported TOTP encoder",
    );
    expect(screen.getByRole("button", { name: "Save" })).toBeDisabled();
  });

  it("rejects a zero period before it reaches the backend", async () => {
    const { container } = render(TotpSetupDialog, {
      onconfirm: vi.fn(),
      oncancel: vi.fn(),
    });
    await scanQr(container, "otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP&period=0");

    expect(await screen.findByRole("alert")).toHaveTextContent(
      "The QR code contains invalid TOTP period",
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
