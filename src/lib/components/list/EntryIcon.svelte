<script lang="ts">
  import type { ItemType } from "$lib/bridge/types";
  import { ENTRY_TYPE_BY_TYPE } from "$lib/entryTypes";
  import Icon from "../ui/Icon.svelte";

  const brandIcons: Record<string, string> = {
    github: "brand-github",
    stripe: "brand-stripe",
    cloudflare: "brand-cloudflare",
    anthropic: "brand-anthropic",
    hetzner: "brand-hetzner",
    tailscale: "brand-tailscale",
    forgejo: "git-branch",
  };

  const brandColors: Record<string, string> = {
    github: "#24292f",
    stripe: "#635bff",
    cloudflare: "#f38020",
    anthropic: "#d97757",
    hetzner: "#d50c2d",
    tailscale: "#202020",
    forgejo: "#4a90d9",
  };

  let {
    iconHint,
    type,
    title,
    size = 30,
  }: {
    iconHint?: string;
    type: ItemType;
    title: string;
    size?: number;
  } = $props();

  let brandKey = $derived(iconHint && brandIcons[iconHint] ? iconHint : null);
  let typeMetadata = $derived(ENTRY_TYPE_BY_TYPE[type]);

  let bgColor = $derived(brandKey ? brandColors[brandKey] : typeMetadata.color);

  let iconName = $derived(brandKey ? brandIcons[brandKey] : typeMetadata.icon);
</script>

<div
  class="entry-icon"
  style:width={`${size}px`}
  style:height={`${size}px`}
  style:background={bgColor}
  style:border-radius={`${Math.round(size * 0.233)}px`}
>
  <Icon name={iconName} size={Math.round(size * 0.53)} />
</div>

<style>
  .entry-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #fff;
  }
</style>
