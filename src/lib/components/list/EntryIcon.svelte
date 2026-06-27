<script lang="ts">
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

  const typeColors: Record<string, string> = {
    login: "#378add",
    note: "#a1a09a",
    identity: "#7f77dd",
    card: "#1d9e75",
  };

  const typeIcons: Record<string, string> = {
    login: "key",
    note: "notes",
    identity: "user",
    card: "credit-card",
  };

  let {
    iconHint,
    type,
    title,
    size = 30,
  }: {
    iconHint?: string;
    type: string;
    title: string;
    size?: number;
  } = $props();

  let brandKey = $derived(iconHint && brandIcons[iconHint] ? iconHint : null);

  let bgColor = $derived(brandKey ? brandColors[brandKey] : typeColors[type] || typeColors.login);

  let iconName = $derived(brandKey ? brandIcons[brandKey] : typeIcons[type] || typeIcons.login);
</script>

<div
  class="entry-icon"
  style="width: {size}px; height: {size}px; background: {bgColor}; border-radius: {Math.round(size * 0.233)}px;"
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
