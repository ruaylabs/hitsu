<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";
  import Button from "./Button.svelte";
  import Icon from "./Icon.svelte";

  type Variant = "ghost" | "outline" | "danger";

  let {
    icon,
    iconSize = 14,
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

  // IconButton is Button's `icon` size; only its danger tone differs in name.
  const buttonVariant = $derived(variant === "danger" ? "ghost-danger" : variant);
</script>

<Button {...rest} variant={buttonVariant} size="icon" {active} class={className}>
  <Icon name={icon} size={iconSize} />
</Button>
