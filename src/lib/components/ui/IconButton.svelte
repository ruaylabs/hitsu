<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Icon from "./Icon.svelte";

  type Variant = "ghost" | "outline" | "danger";

  let {
    icon,
    iconSize = 15,
    variant = "ghost",
    active = false,
    class: className,
    ...rest
  }: HTMLButtonAttributes & {
    icon: string;
    iconSize?: number;
    variant?: Variant;
    active?: boolean;
  } = $props();
</script>

<button
  {...rest}
  class={["icon-button", `icon-button-${variant}`, className].filter(Boolean).join(" ")}
  class:active
>
  <Icon name={icon} size={iconSize} />
</button>

<style>
  .icon-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: var(--icon-button-size);
    height: var(--icon-button-size);
    flex-shrink: 0;
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    transition:
      background var(--transition-fast),
      color var(--transition-fast);
  }

  .icon-button-outline {
    border: 0.5px solid var(--border-strong);
  }

  .icon-button:hover {
    background: var(--border);
  }

  .icon-button-danger:hover {
    color: var(--danger);
    background: var(--danger-bg);
  }

  .icon-button.active {
    color: var(--warning);
  }
</style>
