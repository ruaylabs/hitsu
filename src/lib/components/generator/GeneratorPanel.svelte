<script lang="ts">
  import * as generatorBridge from "$lib/bridge/generator";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";

  let {
    onUse,
    oncancel,
  }: {
    onUse?: (password: string) => void;
    oncancel?: () => void;
  } = $props();

  let length = $state(20);
  let uppercase = $state(true);
  let lowercase = $state(true);
  let digits = $state(true);
  let symbols = $state(false);
  let excludeLookalikes = $state(true);

  let password = $state("");

  async function generate() {
    try {
      password = await generatorBridge.generatePassword({
        length,
        uppercase,
        lowercase,
        digits,
        symbols,
        excludeLookalikes,
      });
    } catch {
      password = "Error generating password";
    }
  }

  $effect(() => {
    generate();
  });
</script>

<Dialog title="Password generator" onclose={oncancel} width="400px">
  {#snippet children()}
    <div class="panel-body">
      <div class="password-display">
        <code class="generated-pw">{password}</code>
        <button
          class="regenerate-btn"
          onclick={generate}
          aria-label="Regenerate"
          title="Regenerate"
        >
          <Icon name="refresh" size={16} />
        </button>
      </div>

      <div class="options">
        <div class="option-row">
          <span class="option-label">Length</span>
          <input type="range" min="8" max="100" bind:value={length} class="range-input" />
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
    <button class="btn btn-cancel" onclick={oncancel}>Cancel</button>
    <button class="btn btn-use" disabled={!password} onclick={() => onUse?.(password)}>
      Use this
    </button>
  {/snippet}
</Dialog>

<style>
  .panel-body {
    padding: 18px;
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

  .generated-pw {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--text-primary);
    word-break: break-all;
    min-width: 0;
  }

  .regenerate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .regenerate-btn:hover {
    background: var(--border);
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

  .btn {
    padding: 6px 14px;
    border-radius: var(--radius-sm);
    font-size: 13px;
  }

  .btn-cancel {
    color: var(--text-secondary);
    background: transparent;
  }

  .btn-cancel:hover {
    background: var(--border);
  }

  .btn-use {
    color: #fff;
    background: var(--accent);
  }

  .btn-use:hover:not(:disabled) {
    opacity: 0.9;
  }

  .btn-use:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
