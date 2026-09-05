<script lang="ts">
  import Icon from "../ui/Icon.svelte";

  let { expiresAt }: { expiresAt: string } = $props();

  function localDateString() {
    const now = new Date();
    const year = now.getFullYear();
    const month = String(now.getMonth() + 1).padStart(2, "0");
    const day = String(now.getDate()).padStart(2, "0");
    return `${year}-${month}-${day}`;
  }

  let due = $derived(expiresAt <= localDateString());

  let label = $derived.by(() => {
    const formatted = new Date(`${expiresAt}T00:00:00`).toLocaleDateString();
    if (expiresAt < localDateString()) return `Expired on ${formatted}`;
    if (expiresAt === localDateString()) return "Expires today";
    return `Expires on ${formatted}`;
  });
</script>

<div class="expiration-indicator" class:due role="status">
  <Icon name={due ? "alert-triangle" : "calendar-time"} size={14} />
  <span>{label}</span>
</div>

<style>
  .expiration-indicator {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: fit-content;
    margin-bottom: 16px;
    padding: var(--space-1) var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: var(--surface-1);
    font-size: var(--text-sm);
  }

  .expiration-indicator.due {
    color: var(--danger);
    background: var(--danger-bg);
  }
</style>
