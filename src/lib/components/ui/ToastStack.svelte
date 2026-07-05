<script lang="ts">
  import { toast } from "$lib/stores/toast.svelte";
  import Icon from "./Icon.svelte";

  const ICONS: Record<string, string> = {
    info: "info-circle",
    success: "check",
    warning: "alert-triangle",
    danger: "alert-triangle",
  };
</script>

{#if toast.all.length > 0}
  <div class="toast-stack" role="status" aria-live="polite">
    {#each toast.all as t (t.id)}
      <button class="toast toast-{t.kind}" onclick={() => toast.dismiss(t.id)} title="Dismiss">
        <Icon name={ICONS[t.kind] ?? "info-circle"} size={16} />
        <span class="toast-message">{t.message}</span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .toast-stack {
    position: fixed;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    z-index: 1100;
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 8px;
    max-width: 440px;
    padding: 10px 14px;
    background: var(--surface-2);
    border: 0.5px solid var(--border-strong);
    border-radius: var(--radius);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
    font-size: 12.5px;
    line-height: 1.4;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
  }

  .toast-danger,
  .toast-warning {
    border-color: var(--danger);
    color: var(--danger);
  }

  .toast-success {
    border-color: var(--success);
  }

  .toast-message {
    min-width: 0;
  }
</style>
