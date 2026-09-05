<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "danger" | "ghost-danger" | "outline" | "ghost" | "link";
  type Size = "xs" | "sm" | "md" | "lg" | "icon";

  let {
    children,
    variant = "secondary",
    size = "sm",
    active = false,
    class: className,
    ...rest
  }: HTMLButtonAttributes & {
    children: Snippet;
    variant?: Variant;
    size?: Size;
    active?: boolean;
  } = $props();
</script>

<button
  {...rest}
  class={["button", `button-${variant}`, `button-${size}`, className].filter(Boolean).join(" ")}
  class:is-active={active}
>
  {@render children()}
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    border-radius: var(--radius-sm);
    transition:
      background var(--transition-fast),
      color var(--transition-fast),
      opacity var(--transition-fast);
  }

  /* Sizes */
  .button-xs {
    gap: var(--space-1);
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-sm);
  }

  .button-sm {
    min-height: 32px;
    padding: var(--space-2) var(--space-4);
    font-size: var(--text-base);
  }

  .button-md {
    padding: var(--space-3) var(--space-4);
    font-size: var(--text-base);
  }

  .button-lg {
    gap: var(--space-3);
    padding: var(--space-3) var(--space-5);
    border-radius: var(--radius);
    font-size: var(--text-base);
    font-weight: 500;
  }

  /* Square icon-only geometry — what IconButton is built on. */
  .button-icon {
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    flex-shrink: 0;
    padding: 0;
  }

  /* Variants — declared after the sizes so a variant can reset size metrics. */
  .button-primary {
    color: var(--text-on-accent);
    background: var(--accent);
  }

  .button-danger {
    color: var(--text-on-accent);
    background: var(--danger);
  }

  .button-secondary,
  .button-ghost {
    color: var(--text-secondary);
    background: transparent;
  }

  .button-ghost-danger {
    color: var(--danger-text);
    background: transparent;
  }

  .button-outline {
    color: var(--text-primary);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
  }

  .button-link {
    min-height: 0;
    padding: 0;
    color: var(--accent);
    background: transparent;
    font-size: var(--text-sm);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  /* Icon buttons sit on chrome rather than on a filled surface, and stay
         neutral until hover — a bare red glyph reads as an error, not an action. */
  .button-icon.button-outline {
    color: var(--text-secondary);
    background: transparent;
    border-color: var(--border-strong);
  }

  .button-icon.button-ghost-danger {
    color: var(--text-secondary);
  }

  .button-primary:hover:not(:disabled),
  .button-danger:hover:not(:disabled) {
    opacity: 0.9;
  }

  .button-secondary:hover:not(:disabled),
  .button-ghost:hover:not(:disabled),
  .button-outline:hover:not(:disabled) {
    background: var(--surface-hover);
  }

  .button-ghost-danger:hover:not(:disabled) {
    background: var(--danger-bg);
  }

  .button-icon.button-ghost-danger:hover:not(:disabled) {
    color: var(--danger);
  }

  .button-link:hover:not(:disabled) {
    color: var(--text-accent);
  }

  .button.is-active {
    color: var(--warning);
  }
</style>
