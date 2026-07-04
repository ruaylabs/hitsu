<script lang="ts">
  import { onMount } from "svelte";
  import { timeAgo, cardBrandName } from "$lib/utils/format";
  import Icon from "../ui/Icon.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import Field from "./Field.svelte";
  import PasswordField from "./PasswordField.svelte";
  import NotesField from "./NotesField.svelte";
  import * as entriesBridge from "$lib/bridge/entries";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import type { HistoryEntrySummary } from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";

  let { entryId, onclose }: { entryId: string; onclose: () => void } = $props();

  let revisions = $state<HistoryEntrySummary[]>([]);
  let loading = $state(true);
  let error = $state("");
  let selectedVersion = $state<number | null>(null);
  let loadingDetail = $state(false);
  let detailEntry = $state<Entry | null>(null);
  let fetchId = 0;

  onMount(async () => {
    loading = true;
    try {
      revisions = await entriesBridge.entryHistoryList(entryId);
      if (revisions.length > 0) {
        selectedVersion = revisions[0].version;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  });

  $effect(() => {
    if (selectedVersion !== null) {
      const thisFetch = ++fetchId;
      // Keep the previous revision's data visible during refetch (mirrors
      // ItemDetail) so switching revisions doesn't flash the fields off/on.
      // Only show "Loading…" when we have no prior data to display.
      if (!detailEntry) loadingDetail = true;
      error = "";
      entriesBridge
        .entryHistoryGet(entryId, selectedVersion)
        .then((e) => {
          if (thisFetch === fetchId) {
            detailEntry = e;
            loadingDetail = false;
          }
        })
        .catch((e) => {
          if (thisFetch === fetchId) {
            error = e instanceof Error ? e.message : String(e);
            loadingDetail = false;
          }
        });
    }
  });

  function formatDate(iso: string): string {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
<div class="overlay" onclick={onclose}>
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="dialog" onclick={(e) => e.stopPropagation()}>
    <div class="dialog-header">
      <h2 class="dialog-title">
        <Icon name="history" size={16} />
        Revision history
      </h2>
      <button class="close-btn" onclick={onclose} aria-label="Close" title="Close">
        <Icon name="x" size={16} />
      </button>
    </div>

    <div class="dialog-body">
      {#if loading}
        <div class="loading">Loading revisions…</div>
      {:else if error}
        <div class="error">{error}</div>
      {:else if revisions.length === 0}
        <div class="empty">No revisions for this entry.</div>
      {:else}
        <div class="revision-layout">
          <div class="revision-list">
            <div class="list-label">Revisions</div>
            {#each revisions as rev}
              <button
                class="revision-row"
                class:selected={selectedVersion === rev.version}
                onclick={() => (selectedVersion = rev.version)}
              >
                <div class="rev-meta">
                  <span class="rev-version">v{revisions.length - rev.version}</span>
                  <span class="rev-date" title={formatDate(rev.modifiedAt)}>
                    {timeAgo(rev.modifiedAt)}
                  </span>
                </div>
                <div class="rev-title">{rev.title}</div>
              </button>
            {/each}
          </div>

          <div class="revision-detail">
            {#if loadingDetail}
              <div class="loading">Loading revision…</div>
            {:else if detailEntry}
              <div class="detail-scroll">
                <div class="detail-title">{detailEntry.title}</div>

                {#if detailEntry.username || detailEntry.password || detailEntry.url}
                  <FieldGroup>
                    {#if detailEntry.username}
                      <Field
                        label="Username"
                        value={detailEntry.username}
                        onCopy={() => clipboard.copyPlain(detailEntry!.username!)}
                      />
                    {/if}
                    {#if detailEntry.password}
                      <PasswordField label="Password" password={detailEntry.password} />
                    {/if}
                    {#if detailEntry.url}
                      <Field
                        label="URL"
                        value={detailEntry.url}
                        onCopy={() => clipboard.copyPlain(detailEntry!.url!)}
                      />
                    {/if}
                  </FieldGroup>
                {/if}

                {#if detailEntry.identity}
                  <FieldGroup>
                    {#if detailEntry.identity.firstName}
                      <Field
                        label="First name"
                        value={detailEntry.identity.firstName}
                        onCopy={() => clipboard.copyPlain(detailEntry!.identity!.firstName!)}
                      />
                    {/if}
                    {#if detailEntry.identity.lastName}
                      <Field
                        label="Last name"
                        value={detailEntry.identity.lastName}
                        onCopy={() => clipboard.copyPlain(detailEntry!.identity!.lastName!)}
                      />
                    {/if}
                    {#if detailEntry.identity.email}
                      <Field
                        label="Email"
                        value={detailEntry.identity.email}
                        onCopy={() => clipboard.copyPlain(detailEntry!.identity!.email!)}
                      />
                    {/if}
                    {#if detailEntry.identity.phone}
                      <Field
                        label="Phone"
                        value={detailEntry.identity.phone}
                        onCopy={() => clipboard.copyPlain(detailEntry!.identity!.phone!)}
                      />
                    {/if}
                    {#if detailEntry.identity.address}
                      <Field
                        label="Address"
                        value={detailEntry.identity.address}
                        onCopy={() => clipboard.copyPlain(detailEntry!.identity!.address!)}
                      />
                    {/if}
                  </FieldGroup>
                {/if}

                {#if detailEntry.card}
                  <FieldGroup>
                    {#if detailEntry.card.type}
                      <Field label="Type" value={cardBrandName(detailEntry.card.type)} />
                    {/if}
                    {#if detailEntry.card.holder}
                      <Field
                        label="Holder"
                        value={detailEntry.card.holder}
                        onCopy={() => clipboard.copyPlain(detailEntry!.card!.holder!)}
                      />
                    {/if}
                    {#if detailEntry.card.number}
                      <Field
                        label="Number"
                        value={detailEntry.card.number}
                        mono
                        onCopy={() => clipboard.copy(detailEntry!.card!.number!)}
                      />
                    {/if}
                    {#if detailEntry.card.expMonth && detailEntry.card.expYear}
                      <Field
                        label="Expires"
                        value={`${String(detailEntry.card.expMonth).padStart(2, "0")}/${detailEntry.card.expYear}`}
                      />
                    {/if}
                    {#if detailEntry.card.cvv}
                      <PasswordField label="CVV" password={detailEntry.card.cvv} />
                    {/if}
                  </FieldGroup>
                {/if}

                {#if detailEntry.tags.length > 0}
                  <div class="tags-display">
                    {#each detailEntry.tags as tag}
                      <span class="tag-badge">{tag}</span>
                    {/each}
                  </div>
                {/if}

                {#if detailEntry.notes}
                  <NotesField notes={detailEntry.notes} />
                {/if}
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.35);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }

  .dialog {
    background: var(--surface-2);
    border-radius: var(--radius-card);
    border: 0.5px solid var(--border);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.18);
    width: min(720px, 90vw);
    height: min(520px, 80vh);
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px;
    border-bottom: 0.5px solid var(--border);
    flex-shrink: 0;
  }

  .dialog-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
    margin: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border: 0.5px solid var(--border);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    background: var(--surface-1);
  }

  .close-btn:hover {
    background: var(--border);
  }

  .dialog-body {
    flex: 1;
    overflow: hidden;
  }

  .loading,
  .error,
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
    color: var(--text-muted);
    padding: 24px;
  }

  .error {
    color: var(--danger);
  }

  .revision-layout {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: 100%;
  }

  .revision-list {
    border-right: 0.5px solid var(--border);
    overflow-y: auto;
    padding: 8px;
  }

  .list-label {
    font-size: 11px;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 6px 8px 8px;
  }

  .revision-row {
    display: block;
    width: 100%;
    padding: 8px 10px;
    border-radius: var(--radius-sm);
    text-align: left;
    margin-bottom: 2px;
  }

  .revision-row:hover {
    background: var(--surface-1);
  }

  .revision-row.selected {
    background: var(--bg-accent);
  }

  .revision-row.selected .rev-version {
    color: var(--text-accent);
  }

  .revision-row.selected .rev-title {
    color: var(--text-primary);
  }

  .rev-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 2px;
  }

  .rev-version {
    font-size: 11px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .rev-date {
    font-size: 11px;
    color: var(--text-muted);
  }

  .rev-title {
    font-size: 12.5px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .revision-detail {
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .detail-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 16px 18px;
  }

  .detail-title {
    font-size: 15px;
    font-weight: 500;
    color: var(--text-primary);
    margin-bottom: 16px;
  }

  .tags-display {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 16px;
  }

  .tag-badge {
    display: inline-block;
    padding: 2px 8px;
    background: var(--surface-1);
    border: 0.5px solid var(--border);
    border-radius: 4px;
    font-size: 11.5px;
    color: var(--text-secondary);
  }
</style>
