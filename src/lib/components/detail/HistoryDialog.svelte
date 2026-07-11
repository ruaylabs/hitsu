<script lang="ts">
  import { onMount, untrack } from "svelte";
  import type { HistoryEntrySummary } from "$lib/bridge/entries";
  import * as entriesBridge from "$lib/bridge/entries";
  import type { Entry } from "$lib/bridge/types";
  import { clipboard } from "$lib/stores/clipboard.svelte";
  import { cardBrandName, formatCardNumber, timeAgo } from "$lib/utils/format";
  import Dialog from "../ui/Dialog.svelte";
  import Icon from "../ui/Icon.svelte";
  import Field from "./Field.svelte";
  import FieldGroup from "./FieldGroup.svelte";
  import NotesField from "./NotesField.svelte";
  import PasswordField from "./PasswordField.svelte";

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

  // Card numbers are shown in full (matching the detail view), fetched per
  // revision alongside the sanitized entry.
  let cardNumberPlain = $state("");

  $effect(() => {
    if (selectedVersion !== null) {
      const thisFetch = ++fetchId;
      const version = selectedVersion;
      // Keep the previous revision's data visible during refetch (mirrors
      // ItemDetail) so switching revisions doesn't flash the fields off/on.
      // Only show "Loading…" when we have no prior data to display.
      // untrack: depending on `detailEntry` here would make the assignment
      // below re-trigger the effect, refetching in a loop on every completion.
      if (!untrack(() => detailEntry)) loadingDetail = true;
      error = "";
      cardNumberPlain = "";
      entriesBridge
        .entryHistoryGet(entryId, version)
        .then((e) => {
          if (thisFetch === fetchId) {
            detailEntry = e;
            loadingDetail = false;
            if (e.card?.hasNumber) {
              entriesBridge
                .entryRevealField(entryId, "cardNumber", version)
                .then((n) => {
                  if (thisFetch === fetchId) cardNumberPlain = n;
                })
                .catch((err) => console.error("Failed to load card number", err));
            }
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
</script>

<Dialog
  title="Revision history"
  {onclose}
  size="lg"
  height="min(520px, 80vh)"
  bodyPadding="none"
  bodyOverflow="hidden"
  bodyFill
>
  {#snippet titleContent()}
    <Icon name="history" size={16} />
    Revision history
  {/snippet}

  {#snippet children()}
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
          {#each revisions as rev (rev.version)}
            <button
              class="revision-row"
              class:selected={selectedVersion === rev.version}
              onclick={() => (selectedVersion = rev.version)}
            >
              <div class="rev-meta">
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

              {#if detailEntry.username || detailEntry.hasPassword || detailEntry.url}
                <FieldGroup>
                  {#if detailEntry.username}
                    <Field
                      label="Username"
                      value={detailEntry.username}
                      onCopy={() => clipboard.copyPlain(detailEntry!.username!)}
                    />
                  {/if}
                  {#if detailEntry.hasPassword}
                    {@const version = selectedVersion ?? undefined}
                    <PasswordField
                      label="Password"
                      reveal={() => entriesBridge.entryRevealField(entryId, "password", version)}
                      copy={() => clipboard.copySecretField(entryId, "password", version)}
                      showStrength
                    />
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
                  {#if detailEntry.card.hasNumber}
                    <Field
                      label="Number"
                      value={cardNumberPlain
                          ? formatCardNumber(cardNumberPlain, detailEntry.card.type)
                          : (detailEntry.card.numberMasked ?? "")}
                      mono
                      onCopy={() =>
                          clipboard.copySecretField(
                            entryId,
                            "cardNumber",
                            selectedVersion ?? undefined,
                          )}
                    />
                  {/if}
                  {#if detailEntry.card.expMonth && detailEntry.card.expYear}
                    <Field
                      label="Expires"
                      value={`${String(detailEntry.card.expMonth).padStart(2, "0")}/${detailEntry.card.expYear}`}
                    />
                  {/if}
                  {#if detailEntry.card.hasCvv}
                    {@const version = selectedVersion ?? undefined}
                    <PasswordField
                      label="CVV"
                      reveal={() => entriesBridge.entryRevealField(entryId, "cardCvv", version)}
                      copy={() => clipboard.copySecretField(entryId, "cardCvv", version)}
                    />
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
  {/snippet}
</Dialog>

<style>
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

  .revision-row.selected .rev-title {
    color: var(--text-primary);
  }

  .rev-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 2px;
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
