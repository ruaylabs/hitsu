<script lang="ts">
  import { createCopyFeedback } from "$lib/utils/copyFeedback.svelte";
  import Button from "../ui/Button.svelte";
  import Icon from "../ui/Icon.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import DetailFieldRow from "./DetailFieldRow.svelte";

  let {
    label,
    value,
    mono = false,
    onOpenUrl,
    onCopy,
    children,
  }: {
    label: string;
    value: string;
    mono?: boolean;
    /** When set, clicking the value opens this URL (http/https only). */
    onOpenUrl?: () => void;
    onCopy?: () => void;
    children?: import("svelte").Snippet;
  } = $props();

  const copied = createCopyFeedback();

  function handleCopy() {
    if (onCopy) void copied.run(onCopy, `${label} copied`);
  }
</script>

<DetailFieldRow {label}>
  {#if onOpenUrl}
    <button class="field-value field-link" class:mono onclick={onOpenUrl} title={value}>
      {value}
    </button>
  {:else}
    <span class="field-value" class:mono>{value}</span>
  {/if}
  {#if children}
    <div class="field-actions">
      {@render children()}
    </div>
  {:else if onCopy}
    <div class="field-actions">
      {#if onOpenUrl}
        <Button variant="ghost" size="xs" onclick={onOpenUrl} aria-label="Open {label}">
          Open<Icon name="external-link" size={12} />
        </Button>
      {/if}
      <IconButton
        icon={copied.active ? "check" : "copy"}
        onclick={handleCopy}
        aria-label="Copy {label}"
        title="Copy {label}"
      />
    </div>
  {/if}
</DetailFieldRow>

<style>
  .field-value {
    font-size: var(--text-base);
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow-wrap: anywhere;
    white-space: pre-wrap;
    user-select: text;
  }

  .field-value.mono {
    font-family: var(--font-mono);
  }

  .field-link {
    color: var(--text-accent);
    text-decoration: none;
    text-align: left;
    width: 100%;
  }

  .field-link:hover {
    text-decoration: underline;
  }

  .field-actions {
    display: flex;
    gap: var(--space-1);
    flex-shrink: 0;
  }
</style>
