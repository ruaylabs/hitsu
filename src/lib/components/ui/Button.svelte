<script lang="ts">
  import type { Snippet } from "svelte";
  import type { HTMLButtonAttributes } from "svelte/elements";

  type Variant = "primary" | "secondary" | "danger" | "outline" | "ghost";
  type Size = "sm" | "md";

  let {
    children,
    variant = "secondary",
    size = "sm",
    class: className,
    ...rest
  }: HTMLButtonAttributes & {
    children: Snippet;
    variant?: Variant;
    size?: Size;
  } = $props();
</script>

<button
  {...rest}
  class={["button", `button-${variant}`, `button-${size}`, className].filter(Boolean).join(" ")}
>
  {@render children()}
</button>

<style>
  .button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border-radius: var(--radius-sm);
    transition:
      background var(--transition-fast),
      opacity var(--transition-fast);
  }

  .button-sm {
    padding: 6px 14px;
    font-size: 13px;
  }

  .button-md {
    padding: 10px 16px;
    font-size: 14px;
  }

  .button-primary {
    color: #fff;
    background: var(--accent);
  }

  .button-danger {
    color: #fff;
    background: var(--danger);
  }

  .button-secondary,
  .button-ghost {
    color: var(--text-secondary);
    background: transparent;
  }

  .button-outline {
    color: var(--text-primary);
    background: var(--surface-1);
    border: 0.5px solid var(--border);
  }

  .button-primary:hover:not(:disabled),
  .button-danger:hover:not(:disabled) {
    opacity: 0.9;
  }

  .button-secondary:hover:not(:disabled),
  .button-ghost:hover:not(:disabled),
  .button-outline:hover:not(:disabled) {
    background: var(--border);
  }

  .button:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
