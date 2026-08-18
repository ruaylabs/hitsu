<script lang="ts">
  import * as generatorBridge from "$lib/bridge/generator";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { createCopyFeedback } from "$lib/utils/copyFeedback.svelte";
  import { errorMessage } from "$lib/utils/errorMessage";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import PasswordStrengthMeter from "../ui/PasswordStrengthMeter.svelte";

  const LENGTH_BOUNDS = { min: 8, max: 100 } as const;
  const OPTIONS_KEY = "hitsu:generator-options";

  interface GeneratorOptions {
    length: number;
    uppercase: boolean;
    lowercase: boolean;
    digits: boolean;
    symbols: boolean;
    excludeLookalikes: boolean;
  }

  let {
    onUse,
    oncancel,
  }: {
    onUse?: (password: string) => void;
    oncancel?: () => void;
  } = $props();

  function loadOptions(): GeneratorOptions | null {
    try {
      const saved = JSON.parse(localStorage.getItem(OPTIONS_KEY) ?? "null");
      if (saved === null) return null;
      return {
        length: Number.isFinite(saved.length) ? saved.length : 20,
        uppercase: Boolean(saved.uppercase),
        lowercase: Boolean(saved.lowercase),
        digits: Boolean(saved.digits),
        symbols: Boolean(saved.symbols),
        excludeLookalikes: Boolean(saved.excludeLookalikes),
      };
    } catch {
      return null;
    }
  }

  let { length, uppercase, lowercase, digits, symbols, excludeLookalikes } = $state(
    loadOptions() ?? {
      length: 20,
      uppercase: true,
      lowercase: true,
      digits: true,
      symbols: false,
      excludeLookalikes: true,
    },
  );

  let password = $state("");
  let error = $state("");
  let hasCharacterSet = $derived(uppercase || lowercase || digits || symbols);

  function persistOptions() {
    try {
      localStorage.setItem(
        OPTIONS_KEY,
        JSON.stringify({ length, uppercase, lowercase, digits, symbols, excludeLookalikes }),
      );
    } catch {
      // Option persistence is optional.
    }
  }

  const copied = createCopyFeedback();

  async function generate() {
    error = "";
    if (!hasCharacterSet) {
      password = "";
      error = "Select at least one character type";
      return;
    }
    try {
      password = await generatorBridge.generatePassword({
        length,
        uppercase,
        lowercase,
        digits,
        symbols,
        excludeLookalikes,
      });
    } catch (generationError) {
      password = "";
      error = errorMessage(generationError) || "Failed to generate a password";
    }
  }

  $effect(() => {
    generate();
  });

  $effect(() => {
    persistOptions();
  });
</script>

<Dialog title="Password generator" onclose={oncancel} size="md">
  {#snippet children()}
    <div class="panel-content">
      <div class="password-display">
        {#if error}
          <span class="generator-error" role="alert">{error}</span>
        {:else}
          <code class="generated-pw">{password}</code>
        {/if}
        <IconButton
          icon="copy"
          iconSize={16}
          disabled={!password}
          onclick={() => copied.run(() => clipboard.copyPlain(password), "Password copied")}
          aria-label="Copy password"
          title="Copy password"
        />
        <IconButton
          icon="refresh"
          iconSize={16}
          onclick={generate}
          aria-label="Regenerate"
          title="Regenerate"
        />
      </div>
      {#if password}
        <PasswordStrengthMeter {password} />
      {/if}

      <div class="options">
        <div class="option-row">
          <span class="option-label">Length</span>
          <input
            type="range"
            min={LENGTH_BOUNDS.min}
            max={LENGTH_BOUNDS.max}
            bind:value={length}
            class="range-input"
          />
          <span class="option-value">{length}</span>
        </div>

        <label class="option-row">
          <span class="option-label">Uppercase</span>
          <input type="checkbox" bind:checked={uppercase} />
        </label>

        <label class="option-row">
          <span class="option-label">Lowercase</span>
          <input type="checkbox" bind:checked={lowercase} />
        </label>

        <label class="option-row">
          <span class="option-label">Digits</span>
          <input type="checkbox" bind:checked={digits} />
        </label>

        <label class="option-row">
          <span class="option-label">Symbols</span>
          <input type="checkbox" bind:checked={symbols} />
        </label>

        <label class="option-row">
          <span class="option-label">Exclude lookalikes</span>
          <input type="checkbox" bind:checked={excludeLookalikes} />
        </label>
      </div>
    </div>
  {/snippet}

  {#snippet footer()}
    <Button onclick={oncancel}>Cancel</Button>
    <Button variant="primary" disabled={!password} onclick={() => onUse?.(password)}>
      Use this
    </Button>
  {/snippet}
</Dialog>

<style>
  .panel-content {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .password-display {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
  }

  .generated-pw,
  .generator-error {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--text-primary);
    word-break: break-all;
    min-width: 0;
  }

  .generator-error {
    color: var(--danger);
    font-family: var(--font-sans);
    font-size: 13px;
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .option-row {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--text-primary);
    cursor: pointer;
  }

  .option-label {
    flex: 1;
  }

  .option-value {
    width: 30px;
    text-align: right;
    color: var(--text-muted);
    font-size: 13px;
  }

  .range-input {
    width: 120px;
    accent-color: var(--accent);
  }
</style>
