<script lang="ts">
  import * as generatorBridge from "$lib/bridge/generator";
  import Button from "../ui/Button.svelte";
  import Dialog from "../ui/Dialog.svelte";
  import IconButton from "../ui/IconButton.svelte";

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

<Dialog title="Password generator" onclose={oncancel} size="md">
  {#snippet children()}
    <div class="panel-content">
      <div class="password-display">
        <code class="generated-pw">{password}</code>
        <IconButton
          icon="refresh"
          iconSize={16}
          onclick={generate}
          aria-label="Regenerate"
          title="Regenerate"
        />
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

  .generated-pw {
    flex: 1;
    font-family: var(--font-mono);
    font-size: 14px;
    color: var(--text-primary);
    word-break: break-all;
    min-width: 0;
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
