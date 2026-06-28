<script lang="ts">
  import { app } from "$lib/stores/app.svelte";
  import { vault } from "$lib/stores/vault.svelte";
  import { selection } from "$lib/stores/selection.svelte";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import Icon from "../ui/Icon.svelte";

  let itemCount = $derived(vault.entries.length);

  async function createEntry(type: string) {
    try {
      const entry = await entriesBridge.entryCreate(type, { title: `New ${type}` });
      vault.setEntries([...vault.entries, entry]);
      selection.selectedId = entry.id;
      vault.setEditingId(entry.id);
    } catch (e) {
      console.error("Failed to create entry", e);
    }
  }

  let showTypePicker = $state(false);

  $effect(() => {
    if (!showTypePicker) return;
    const close = () => {
      showTypePicker = false;
    };
    document.addEventListener("click", close);
    return () => document.removeEventListener("click", close);
  });
</script>

<footer class="statusbar">
  <div class="statusbar-left">
    <span class="sync-dot"></span>
    <span>Changes saved</span>
  </div>
  <div class="statusbar-right">
    {#if showTypePicker}
      <div class="type-picker" role="menu">
        <button
          class="type-item"
          onclick={() => { showTypePicker = false; createEntry("login"); }}
          role="menuitem"
        >
          <Icon name="key" size={12} />
          Login
        </button>
        <button
          class="type-item"
          onclick={() => { showTypePicker = false; createEntry("note"); }}
          role="menuitem"
        >
          <Icon name="notes" size={12} />
          Note
        </button>
        <button
          class="type-item"
          onclick={() => { showTypePicker = false; createEntry("identity"); }}
          role="menuitem"
        >
          <Icon name="user" size={12} />
          Identity
        </button>
        <button
          class="type-item"
          onclick={() => { showTypePicker = false; createEntry("card"); }}
          role="menuitem"
        >
          <Icon name="credit-card" size={12} />
          Card
        </button>
      </div>
    {/if}
    {#if clipboard.active}
      <span class="countdown">Clears in {Math.ceil(clipboard.remainingMs / 1000)}s</span>
    {:else}
      <span>{itemCount} items</span>
    {/if}
    {#if vault.meta}
      <button
        class="add-btn"
        onclick={(e) => { e.stopPropagation(); showTypePicker = !showTypePicker; }}
        aria-label="Add entry"
      >
        <Icon name="plus" size={12} />
      </button>
      <button class="lock-btn" onclick={() => vault.lock()} aria-label="Lock vault">
        <Icon name="lock" size={12} />
      </button>
    {/if}
    <button class="settings-gear" onclick={() => app.toggleSettings()} aria-label="Settings">
      <Icon name="settings" size={12} />
    </button>
  </div>
</footer>

<style>
  .statusbar {
    height: var(--statusbar-height);
    padding: 0 14px;
    background: var(--surface-1);
    border-top: 0.5px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 11.5px;
    color: var(--text-muted);
    position: relative;
  }

  .statusbar-left {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .sync-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--success);
    flex-shrink: 0;
  }

  .statusbar-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .add-btn,
  .lock-btn,
  .settings-gear {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 3px;
    color: var(--text-muted);
  }

  .add-btn:hover,
  .lock-btn:hover,
  .settings-gear:hover {
    background: var(--border);
    color: var(--text-secondary);
  }

  .type-picker {
    position: absolute;
    bottom: 100%;
    right: 14px;
    margin-bottom: 4px;
    background: var(--surface-2);
    border: 0.5px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
    display: flex;
    flex-direction: column;
    padding: 4px;
    z-index: 50;
  }

  .type-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border-radius: var(--radius-sm);
    font-size: 13px;
    color: var(--text-primary);
    text-align: left;
    white-space: nowrap;
  }

  .type-item:hover {
    background: var(--bg-accent);
  }
</style>
