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
      <div class="toast toast-{t.kind}">
        <Icon name={ICONS[t.kind] ?? "info-circle"} size={16} />
        <span class="toast-message">{t.message}</span>
        {#if t.action}
          <button
            type="button"
            class="toast-action"
            onclick={() => {
              toast.dismiss(t.id);
              void t.action?.run();
            }}
          >
            {t.action.label}
          </button>
        {/if}
        <button
          type="button"
          class="toast-dismiss"
          onclick={() => toast.dismiss(t.id)}
          aria-label="Dismiss notification"
          title="Dismiss"
        >
          <Icon name="x" size={14} />
        </button>
      </div>
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
    box-shadow: var(--shadow-dialog);
    font-size: 12.5px;
    line-height: 1.4;
    color: var(--text-primary);
    text-align: left;
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

  .toast-action {
    flex-shrink: 0;
    color: var(--accent);
    font-weight: 600;
  }

  .toast-action:hover {
    text-decoration: underline;
  }

  .toast-dismiss {
    display: inline-flex;
    flex-shrink: 0;
    color: var(--text-muted);
  }

  .toast-dismiss:hover {
    color: var(--text-primary);
  }
</style>
