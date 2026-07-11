<script lang="ts">
  import IconButton from "../ui/IconButton.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";

  let {
    label,
    value,
    reveal = false,
    mono = false,
    onOpenUrl,
    onCopy,
    onReveal,
    children,
  }: {
    label: string;
    value: string;
    reveal?: boolean;
    mono?: boolean;
    /** When set, clicking the value opens this URL (http/https only). */
    onOpenUrl?: () => void;
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

<DetailFieldRow {label}>
  {#if onOpenUrl}
    <button class="field-value field-link" class:mono onclick={onOpenUrl} title={value}>
      {displayValue}
    </button>
  {:else}
    <span class="field-value" class:mono>{displayValue}</span>
  {/if}
  {#if children}
    <div class="field-actions">
      {@render children()}
    </div>
  {:else if onCopy}
    <div class="field-actions">
      <IconButton
        icon={copied ? "check" : "copy"}
        onclick={handleCopy}
        aria-label="Copy {label}"
        title="Copy {label}"
      />
      {#if label.toLowerCase().includes("password") && onReveal}
        <IconButton
          icon={reveal ? "eye-off" : "eye"}
          onclick={onReveal}
          aria-label="Reveal {label}"
          title="Reveal {label}"
        />
      {/if}
    </div>
  {/if}
</DetailFieldRow>

<style>
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

  .field-link {
    color: var(--text-accent);
    text-decoration: none;
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    text-align: left;
    width: 100%;
  }

  .field-link:hover {
    text-decoration: underline;
  }

  .field-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
</style>
