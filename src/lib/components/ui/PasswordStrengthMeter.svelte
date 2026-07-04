<script lang="ts">
  import { estimateStrength, strengthColor } from "$lib/utils/passwordStrength";

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
</script>

{#if visible}
  <div class="strength-meter" aria-hidden="true">
    <div class="strength-bar">
      <div
        class="strength-fill"
        style="width: {password ? strength.fraction * 100 : 0}%; background: {strengthColor(strength.level)};"
      ></div>
    </div>
    {#if password}
      <span class="strength-label" style="color: {strengthColor(strength.level)}">
        {strength.label}
      </span>
    {/if}
  </div>
{/if}

<style>
  .strength-meter {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
    min-height: 14px;
  }

  .strength-bar {
    flex: 1;
    height: 6px;
    background: var(--border-strong);
    border-radius: 3px;
    overflow: hidden;
  }

  .strength-fill {
    height: 100%;
    border-radius: 3px;
    transition:
      width 0.15s ease,
      background 0.15s ease;
  }

  .strength-label {
    font-size: 11px;
    font-weight: 500;
    white-space: nowrap;
    min-width: 56px;
    text-align: right;
  }
</style>
