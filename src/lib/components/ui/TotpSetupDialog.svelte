<script lang="ts">
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";

  let {
    oncancel,
    onconfirm,
  }: {
    oncancel: () => void;
    onconfirm: (otpauthUri: string) => void;
  } = $props();

  let seed = $state("");
  let error = $state("");

  const BASE32_RE = /^[A-Z2-7]+=*$/i;

  function normalizeSeed(raw: string): string {
    // Remove whitespace and dashes, uppercase
    return raw.replace(/[\s-]/g, "").toUpperCase();
  }

  function isValidBase32(s: string): boolean {
    return s.length > 0 && BASE32_RE.test(s);
  }

  function submit() {
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
    const uri = `otpauth://totp/entry?secret=${encodeURIComponent(normalized)}&period=30&digits=6`;
    onconfirm(uri);
  }
</script>

<Dialog
  title="Setup TOTP from seed"
  onclose={oncancel}
  onconfirm={submit}
  size="md"
  closeLabel="Cancel"
>
  {#snippet children()}
    <div class="dialog-content">
      <p class="dialog-message">
        Enter the TOTP secret seed code from the website. It will be converted to the standard
        otpauth:// format and saved to this entry.
      </p>
      <label class="control-label" for="totp-seed">Seed code</label>
      <!-- svelte-ignore a11y_autofocus -->
      <input
        id="totp-seed"
        type="text"
        class="control control--mono"
        aria-invalid={Boolean(error)}
        placeholder="e.g. JBSWY3DPEHPK3PXP"
        autofocus
        autocomplete="off"
        autocorrect="off"
        autocapitalize="characters"
        spellcheck="false"
        bind:value={seed}
        oninput={() => { error = ""; }}
      />
      {#if error}
        <span class="control-error">{error}</span>
      {/if}
    </div>
  {/snippet}

  {#snippet footer()}
    <Button onclick={oncancel}>Cancel</Button>
    <Button variant="primary" onclick={submit} disabled={!seed.trim()}>Save</Button>
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
    margin: 0;
  }
</style>
