<script lang="ts">
  import Icon from "../ui/Icon.svelte";

  let {
    label,
    value,
    reveal = false,
    mono = false,
    onCopy,
    onReveal,
    children,
  }: {
    label: string;
    value: string;
    reveal?: boolean;
    mono?: boolean;
    onCopy?: () => void;
    onReveal?: () => void;
    children?: import("svelte").Snippet;
  } = $props();

  let visibleValue = $derived(reveal ? value : value);
  let displayValue = $derived(
    label.toLowerCase().includes("password") && !reveal ? "•".repeat(14) : visibleValue,
  );

  let copied = $state(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;

  function handleCopy() {
    onCopy?.();
    if (copyTimer) clearTimeout(copyTimer);
    copied = true;
    copyTimer = setTimeout(() => (copied = false), 1000);
  }
</script>

<div class="field-row">
  <span class="field-label">{label}</span>
  <span class="field-value" class:mono>{displayValue}</span>
  {#if children}
    <div class="field-actions">
      {@render children()}
    </div>
  {:else if onCopy}
    <div class="field-actions">
      <button
        class="field-action"
        onclick={handleCopy}
        aria-label="Copy {label}"
        title="Copy {label}"
      >
        <Icon name={copied ? "check" : "copy"} size={15} />
      </button>
      {#if label.toLowerCase().includes("password") && onReveal}
        <button
          class="field-action"
          onclick={onReveal}
          aria-label="Reveal {label}"
          title="Reveal {label}"
        >
          <Icon name={reveal ? "eye-off" : "eye"} size={15} />
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .field-row {
    background: var(--surface-2);
    padding: 10px 12px;
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 38px;
  }

  .field-label {
    font-size: 11px;
    color: var(--text-muted);
    width: 70px;
    flex-shrink: 0;
  }

  .field-value {
    font-size: 13.5px;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .field-value.mono {
    font-family: var(--font-mono);
  }

  .field-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .field-action {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition: background 0.1s;
  }

  .field-action:hover {
    background: var(--border);
  }
</style>
