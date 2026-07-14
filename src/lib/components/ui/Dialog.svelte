<script lang="ts">
  import { onMount, type Snippet } from "svelte";
  import IconButton from "./IconButton.svelte";

  let {
    title,
    onclose,
    onconfirm,
    onkeydown,
    children,
    footer,
    showFooter = true,
    titleContent,
    showHeader = true,
    placement = "center",
    topOffset = "96px",
    size = "md",
    width,
    height,
    maxWidth = "90vw",
    maxHeight,
    bodyPadding = "normal",
    bodyOverflow = "visible",
    bodyMaxHeight,
    bodyFill = false,
    transparent = false,
    closeLabel = "Close",
  }: {
    title: string;
    onclose?: () => void;
    onconfirm?: () => void;
    onkeydown?: (event: KeyboardEvent) => void;
    children: Snippet;
    footer?: Snippet;
    showFooter?: boolean;
    titleContent?: Snippet;
    showHeader?: boolean;
    placement?: "center" | "top";
    topOffset?: string;
    size?: "sm" | "md" | "lg";
    width?: string;
    height?: string;
    maxWidth?: string;
    maxHeight?: string;
    bodyPadding?: "normal" | "compact" | "none";
    bodyOverflow?: "visible" | "auto" | "hidden";
    bodyMaxHeight?: string;
    bodyFill?: boolean;
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
      const autofocusTarget = pane.querySelector<HTMLElement>("[autofocus]:not(:disabled)");
      if (autofocusTarget) autofocusTarget.focus();
      else if (!pane.contains(document.activeElement)) pane.focus();
    });

    return () => previouslyFocused?.focus();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div
  class="dialog-overlay"
  class:transparent
  class:placement-top={placement === "top"}
  style:--dialog-top-offset={topOffset}
  onclick={handleBackdropClick}
  role="presentation"
>
  <div
    bind:this={pane}
    class={["dialog-pane", `dialog-${size}`].join(" ")}
    role="dialog"
    aria-modal="true"
    aria-label={title}
    tabindex="-1"
    style:width
    style:height
    style:max-width={maxWidth}
    style:max-height={maxHeight}
    {onkeydown}
  >
    {#if showHeader}
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
    {/if}

    <div
      class="dialog-body"
      class:body-fill={bodyFill}
      class:padding-compact={bodyPadding === "compact"}
      class:padding-none={bodyPadding === "none"}
      style:overflow={bodyOverflow}
      style:max-height={bodyMaxHeight}
    >
      {@render children()}
    </div>

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

  .dialog-overlay.placement-top {
    align-items: flex-start;
    padding-top: var(--dialog-top-offset);
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

  .dialog-sm {
    width: 360px;
  }

  .dialog-md {
    width: 400px;
  }

  .dialog-lg {
    width: 720px;
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

  .dialog-body {
    padding: 20px;
  }

  .dialog-body.padding-compact {
    padding: 12px;
  }

  .dialog-body.padding-none {
    padding: 0;
  }

  .dialog-body.body-fill {
    flex: 1;
    min-height: 0;
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
