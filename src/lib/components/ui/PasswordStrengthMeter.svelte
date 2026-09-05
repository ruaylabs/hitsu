<script lang="ts">
  import { estimateStrength, STRENGTH_LABELS, strengthColor } from "$lib/utils/passwordStrength";

  let {
    password,
    showWhenEmpty = false,
  }: {
    password: string;
    /** Show the (empty) track even when the password is blank. */
    showWhenEmpty?: boolean;
  } = $props();

  let strength = $derived.by(() => estimateStrength(password));
  let visible = $derived(showWhenEmpty || password.length > 0);
  let statusLabel = $derived(
    password ? `Password strength: ${strength.label}` : "Password strength: not yet evaluated",
  );
</script>

{#if visible}
  <div class="strength-meter" role="status" aria-live="polite" aria-label={statusLabel}>
    <div class="strength-bar" aria-hidden="true">
      <div
        class="strength-fill"
        style="width: {password ? strength.fraction * 100 : 0}%; background: {strengthColor(strength.level)};"
      ></div>
    </div>
    {#if password}
      <span class="strength-label" style="color: {strengthColor(strength.level)}">
        <span>Strength: {strength.label}</span>
        <!-- Reserves the width of the longest label so the bar doesn't
             resize as the strength changes. -->
        {#each STRENGTH_LABELS as label (label)}
          <span class="strength-label-sizer" aria-hidden="true">Strength: {label}</span>
        {/each}
      </span>
    {/if}
  </div>
{/if}

<style>
  .strength-meter {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: 2px;
    min-height: 14px;
  }

  .strength-bar {
    flex: 1;
    height: 6px;
    background: var(--border-strong);
    border-radius: var(--radius-xs);
    overflow: hidden;
  }

  .strength-fill {
    height: 100%;
    border-radius: var(--radius-xs);
    transition:
      width var(--transition-base) ease,
      background var(--transition-base) ease;
  }

  .strength-label {
    display: grid;
    justify-items: end;
    font-size: var(--text-sm);
    font-weight: 500;
    white-space: nowrap;
  }

  /* Every label stacked in one grid cell: the column takes the width of the
     widest, and only the live one is visible. */
  .strength-label > span {
    grid-area: 1 / 1;
  }

  .strength-label-sizer {
    visibility: hidden;
  }
</style>
