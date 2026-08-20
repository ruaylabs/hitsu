<script lang="ts">
  import jsQR from "jsqr";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";

  let {
    oncancel,
    onconfirm,
  }: {
    oncancel: () => void;
    onconfirm: (otpauthUri: string) => void;
  } = $props();

  let mode = $state<"qr" | "seed">("qr");
  let seed = $state("");
  let qrUri = $state("");
  let qrLabel = $state("");
  let error = $state("");
  let scanning = $state(false);
  let fileInput = $state<HTMLInputElement>();

  const BASE32_RE = /^[A-Z2-7]+=*$/i;
  const SUPPORTED_ALGORITHMS = new Set(["SHA1", "SHA256", "SHA512"]);
  const MAX_IMAGE_BYTES = 10 * 1024 * 1024;
  const MAX_SCAN_DIMENSION = 2048;
  const MAX_TOTP_PERIOD = 24 * 60 * 60;
  const MAX_TOTP_DIGITS = 10;

  function normalizeSeed(raw: string): string {
    return raw.replace(/[\s-]/g, "").toUpperCase();
  }

  function isValidBase32(value: string): boolean {
    return value.length > 0 && BASE32_RE.test(value);
  }

  function integerParameter(
    uri: URL,
    name: "period" | "digits",
    fallback: number,
    maximum: number,
  ): number {
    const raw = uri.searchParams.get(name);
    if (raw === null) return fallback;
    if (!/^\d+$/.test(raw)) throw new Error(`The QR code contains invalid TOTP ${name}`);

    const value = Number(raw);
    if (!Number.isSafeInteger(value) || value < 1 || value > maximum) {
      throw new Error(`The QR code contains invalid TOTP ${name}`);
    }
    return value;
  }

  function validateOtpAuthUri(raw: string): { uri: string; label: string } {
    let uri: URL;
    try {
      uri = new URL(raw.trim());
    } catch {
      throw new Error("The QR code does not contain a valid TOTP setup link");
    }

    if (uri.protocol !== "otpauth:" || uri.hostname.toLowerCase() !== "totp") {
      throw new Error("The QR code is not a TOTP setup code");
    }

    const secret = normalizeSeed(uri.searchParams.get("secret") ?? "");
    if (!isValidBase32(secret)) {
      throw new Error("The QR code contains an invalid TOTP secret");
    }

    const period = integerParameter(uri, "period", 30, MAX_TOTP_PERIOD);
    const digits = integerParameter(uri, "digits", 6, MAX_TOTP_DIGITS);
    const rawAlgorithm = uri.searchParams.get("algorithm");
    const algorithm = (rawAlgorithm ?? "SHA1").toUpperCase().replaceAll("-", "");
    if (!SUPPORTED_ALGORITHMS.has(algorithm)) {
      throw new Error("The QR code uses an unsupported TOTP algorithm");
    }
    const encoder = uri.searchParams.get("encoder");
    if (encoder && encoder.toLowerCase() !== "numeric") {
      throw new Error("The QR code uses an unsupported TOTP encoder");
    }

    const labelPath = uri.pathname.replace(/^\/+/, "") || "entry";
    let label: string;
    try {
      label = decodeURIComponent(labelPath);
    } catch {
      throw new Error("The QR code contains an invalid TOTP account label");
    }
    const issuer = uri.searchParams.get("issuer")?.trim() ?? "";
    const params = new URLSearchParams({
      secret,
      period: String(period),
      digits: String(digits),
    });
    if (issuer) params.set("issuer", issuer);
    if (rawAlgorithm) params.set("algorithm", algorithm);

    return {
      uri: `otpauth://totp/${labelPath}?${params}`,
      label: label || issuer || "TOTP account",
    };
  }

  async function scanQrCode(file: File) {
    error = "";
    qrUri = "";
    qrLabel = "";

    if (!file.type.startsWith("image/")) {
      error = "Choose an image containing a TOTP QR code";
      return;
    }
    if (file.size > MAX_IMAGE_BYTES) {
      error = "The image is too large (maximum 10 MB)";
      return;
    }

    scanning = true;
    try {
      const bitmap = await createImageBitmap(file);
      const scale = Math.min(1, MAX_SCAN_DIMENSION / Math.max(bitmap.width, bitmap.height));
      const width = Math.max(1, Math.round(bitmap.width * scale));
      const height = Math.max(1, Math.round(bitmap.height * scale));
      const canvas = document.createElement("canvas");
      canvas.width = width;
      canvas.height = height;
      const context = canvas.getContext("2d", { willReadFrequently: true });
      if (!context) {
        bitmap.close();
        throw new Error("Could not read the selected image");
      }

      context.drawImage(bitmap, 0, 0, width, height);
      bitmap.close();
      const image = context.getImageData(0, 0, width, height);
      const result = jsQR(image.data, image.width, image.height, {
        inversionAttempts: "attemptBoth",
      });
      if (!result) throw new Error("No QR code was found in the image");

      const validated = validateOtpAuthUri(result.data);
      qrUri = validated.uri;
      qrLabel = validated.label;
    } catch (scanError) {
      error = scanError instanceof Error ? scanError.message : "Could not scan the QR code";
    } finally {
      scanning = false;
      if (fileInput) fileInput.value = "";
    }
  }

  function handleFile(file: File | undefined) {
    if (file) void scanQrCode(file);
  }

  function createDefaultTotpUri(secret: string): string {
    return `otpauth://totp/entry?secret=${encodeURIComponent(secret)}&period=30&digits=6`;
  }

  function submit() {
    if (mode === "qr") {
      if (!qrUri) {
        error = "Import a TOTP QR code first";
        return;
      }
      onconfirm(qrUri);
      return;
    }

    const normalized = normalizeSeed(seed);
    if (!normalized) {
      error = "Seed code is required";
      return;
    }
    if (!isValidBase32(normalized)) {
      error = "Invalid seed code (expected base32 characters: A-Z, 2-7)";
      return;
    }
    error = "";
    onconfirm(createDefaultTotpUri(normalized));
  }

  function selectMode(nextMode: "qr" | "seed") {
    mode = nextMode;
    error = "";
  }
</script>

<Dialog title="Setup TOTP" onclose={oncancel} onconfirm={submit} size="md" closeLabel="Cancel">
  {#snippet children()}
    <div class="dialog-content">
      <div class="mode-switch" role="tablist" aria-label="TOTP setup method">
        <button
          type="button"
          class:active={mode === "qr"}
          role="tab"
          aria-selected={mode === "qr"}
          onclick={() => selectMode("qr")}
        >
          QR code
        </button>
        <button
          type="button"
          class:active={mode === "seed"}
          role="tab"
          aria-selected={mode === "seed"}
          onclick={() => selectMode("seed")}
        >
          Seed code
        </button>
      </div>

      {#if mode === "qr"}
        <p class="dialog-message">
          Import the QR code shown by the website. The image is scanned locally and is never
          uploaded.
        </p>
        <input
          bind:this={fileInput}
          id="totp-qr-file"
          class="file-input"
          type="file"
          accept="image/*"
          onchange={(event) => handleFile(event.currentTarget.files?.[0])}
        />
        <!-- svelte-ignore a11y_autofocus -->
        <button
          type="button"
          class="image-picker"
          class:image-picker--success={Boolean(qrUri)}
          disabled={scanning}
          autofocus
          onclick={() => fileInput?.click()}
        >
          <i class={`ti ti-${qrUri ? "circle-check" : "photo-scan"}`} aria-hidden="true"></i>
          {#if scanning}
            <strong>Scanning image…</strong>
          {:else if qrUri}
            <strong>QR code ready</strong>
            <span>{qrLabel}</span>
          {:else}
            <strong>Choose a QR code image</strong>
            <span>PNG, JPEG, WebP, or GIF</span>
          {/if}
        </button>
      {:else}
        <p class="dialog-message">
          Enter the TOTP secret seed code from the website. It will be converted to the standard
          otpauth:// format and saved to this entry.
        </p>
        <label class="control-label" for="totp-seed">Seed code</label>
        <input
          id="totp-seed"
          type="text"
          class="control control--mono"
          aria-invalid={Boolean(error)}
          placeholder="e.g. JBSWY3DPEHPK3PXP"
          autocomplete="off"
          autocorrect="off"
          autocapitalize="characters"
          spellcheck="false"
          bind:value={seed}
          oninput={() => { error = ""; }}
        />
      {/if}

      {#if error}
        <span class="control-error" role="alert">{error}</span>
      {/if}
    </div>
  {/snippet}

  {#snippet footer()}
    <Button onclick={oncancel}>Cancel</Button>
    <Button
      variant="primary"
      onclick={submit}
      disabled={scanning || (mode === "qr" ? !qrUri : !seed.trim())}
      >Save</Button
    >
  {/snippet}
</Dialog>

<style>
  .dialog-content {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .dialog-message {
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .mode-switch {
    display: grid;
    grid-template-columns: 1fr 1fr;
    padding: 2px;
    border-radius: var(--radius-sm);
    background: var(--surface-1);
  }

  .mode-switch button {
    min-height: 30px;
    border-radius: calc(var(--radius-sm) - 2px);
    color: var(--text-secondary);
    font-size: 13px;
  }

  .mode-switch button.active {
    color: var(--text-primary);
    background: var(--surface-2);
    box-shadow: 0 1px 3px var(--border);
  }

  .file-input {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .image-picker {
    display: flex;
    min-height: 132px;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 20px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: var(--surface-1);
    transition:
      border-color var(--transition-fast),
      background var(--transition-fast);
  }

  .image-picker:hover:not(:disabled) {
    border-color: var(--accent);
    background: var(--bg-accent);
  }

  .image-picker--success {
    border-style: solid;
    border-color: var(--success);
  }

  .image-picker .ti {
    color: var(--accent);
    font-size: 28px;
  }

  .image-picker--success .ti {
    color: var(--success);
  }

  .image-picker strong {
    color: var(--text-primary);
    font-size: 13px;
    font-weight: 500;
  }

  .image-picker span {
    font-size: 12px;
  }
</style>
