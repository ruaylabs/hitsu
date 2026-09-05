<script lang="ts">
  import { entryGetCustomIcon } from "$lib/bridge/entries";
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
    customIconData = undefined,
    hasCustomIcon = false,
    entryId = undefined,
  }: {
    iconHint?: string;
    type: ItemType;
    title: string;
    size?: number;
    customIconData?: string;
    hasCustomIcon?: boolean;
    entryId?: string;
  } = $props();

  let brandKey = $derived(iconHint && brandIcons[iconHint] ? iconHint : null);
  let typeMetadata = $derived(ENTRY_TYPE_BY_TYPE[type]);

  let tileColor = $derived(brandKey ? brandColors[brandKey] : typeMetadata.color);
  let iconName = $derived(brandKey ? brandIcons[brandKey] : typeMetadata.icon);

  let lazyIconData = $state<string | null>(null);

  $effect(() => {
    if (hasCustomIcon && entryId && !customIconData) {
      entryGetCustomIcon(entryId)
        .then((data) => {
          lazyIconData = data;
        })
        .catch(() => {
          lazyIconData = null;
        });
    }
  });

  let resolvedIcon = $derived(customIconData || lazyIconData);
</script>

<!-- Keep the tile's squircle ratio proportional at list and detail sizes. -->
{#if resolvedIcon}
  <img
    class="entry-icon entry-icon--image"
    style:width={`${size}px`}
    style:height={`${size}px`}
    style:border-radius={`${Math.round(size * 0.233)}px`}
    src={resolvedIcon}
    alt=""
  />
{:else}
  <div
    class="entry-icon"
    class:entry-icon--brand={brandKey !== null}
    style:--entry-icon-color={tileColor}
    style:width={`${size}px`}
    style:height={`${size}px`}
    style:border-radius={`${Math.round(size * 0.233)}px`}
  >
    <Icon name={iconName} size={Math.round(size * 0.53)} />
  </div>
{/if}

<style>
  .entry-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: #fff;
    background: var(--entry-icon-color);
    border: 0.5px solid color-mix(in srgb, currentColor 20%, transparent);
  }

  .entry-icon--brand {
    color: color-mix(in srgb, var(--entry-icon-color) 50%, var(--text-primary));
    background: color-mix(in srgb, var(--entry-icon-color) 14%, transparent);
  }

  .entry-icon--image {
    object-fit: cover;
  }
</style>
