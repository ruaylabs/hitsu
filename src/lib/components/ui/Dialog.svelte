<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";

  let {
    title,
    onclose,
    onconfirm,
    children,
    footer,
    showFooter = true,
    titleContent,
    width = "380px",
    height,
    maxWidth = "90vw",
    maxHeight,
    transparent = false,
    closeLabel = "Close",
  }: {
    title: string;
    onclose?: () => void;
    onconfirm?: () => void;
    children: Snippet;
    footer?: Snippet;
    showFooter?: boolean;
    titleContent?: Snippet;
    width?: string;
    height?: string;
    maxWidth?: string;
    maxHeight?: string;
    transparent?: boolean;
    closeLabel?: string;
  } = $props();

  let pane: HTMLDivElement;

  function handleBackdropClick(event: MouseEvent) {
    if (event.target === event.currentTarget) onclose?.();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Tab") {
      const focusable = Array.from(
        pane.querySelectorAll<HTMLElement>(
          'button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled), a[href], [tabindex]:not([tabindex="-1"])',
        ),
      ).filter((element) => element.offsetParent !== null);

      if (focusable.length === 0) {
        event.preventDefault();
        pane.focus();
      } else if (event.shiftKey && document.activeElement === focusable[0]) {
        event.preventDefault();
        focusable.at(-1)?.focus();
      } else if (!event.shiftKey && document.activeElement === focusable.at(-1)) {
        event.preventDefault();
        focusable[0].focus();
      }
    } else if (event.key === "Escape" && onclose) {
      event.preventDefault();
      onclose();
    } else if (event.key === "Enter" && onconfirm) {
      event.preventDefault();
      onconfirm();
    }
  }

  onMount(() => {
    const previouslyFocused = document.activeElement as HTMLElement | null;
    queueMicrotask(() => {
      if (!pane.contains(document.activeElement)) pane.focus();
    });

    return () => previouslyFocused?.focus();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="dialog-overlay" class:transparent onclick={handleBackdropClick} role="presentation">
  <div
    bind:this={pane}
    class="dialog-pane"
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    style:width
    style:height
    style:max-width={maxWidth}
    style:max-height={maxHeight}
  >
    <header class="dialog-header">
      <h2 class="dialog-title">
        {#if titleContent}
          {@render titleContent()}
        {:else}
          {title}
        {/if}
      </h2>
      {#if onclose}
        <IconButton
          icon="x"
          iconSize={16}
          onclick={onclose}
          aria-label={closeLabel}
          title={closeLabel}
        />
      {/if}
    </header>

    {@render children()}

    {#if footer && showFooter}
      <footer class="dialog-footer">
        {@render footer()}
      </footer>
    {/if}
  </div>
</div>

<style>
  .dialog-overlay {
    position: fixed;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--backdrop);
    z-index: var(--z-dialog);
  }

  .dialog-overlay.transparent {
    background: transparent;
    pointer-events: none;
  }

  .dialog-overlay.transparent .dialog-pane {
    pointer-events: auto;
  }

  .dialog-pane {
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    box-shadow: var(--shadow-dialog);
    outline: none;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex-shrink: 0;
    padding: 16px 20px;
    border-bottom: 0.5px solid var(--border);
  }

  .dialog-title {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    color: var(--text-primary);
    font-size: 15px;
    font-weight: 500;
  }

  .dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    flex-shrink: 0;
    padding: 12px 20px;
    border-top: 0.5px solid var(--border);
  }
</style>
