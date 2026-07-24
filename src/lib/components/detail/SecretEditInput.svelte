<script lang="ts">
  import type { HTMLInputAttributes } from "svelte/elements";
  import Icon from "../ui/Icon.svelte";

  let {
    value = $bindable(),
    label,
    placeholder,
    inputmode,
    pattern,
    maxlength,
    invalid = false,
    sanitize,
    class: className,
  }: {
    value: string;
    label: string;
    placeholder?: string;
    inputmode?: HTMLInputAttributes["inputmode"];
    pattern?: string;
    maxlength?: number;
    invalid?: boolean;
    sanitize?: (value: string) => string;
    class?: string;
  } = $props();

  let revealed = $state(false);

  function handleInput(event: Event) {
    const input = event.currentTarget as HTMLInputElement;
    value = sanitize ? sanitize(input.value) : input.value;
    if (input.value !== value) input.value = value;
  }
</script>

<div class={["secret-edit-input", className].filter(Boolean).join(" ")}>
  <input
    class="control control--compact"
    type={revealed ? "text" : "password"}
    {placeholder}
    {inputmode}
    {pattern}
    {maxlength}
    aria-label={label}
    aria-invalid={invalid}
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    {value}
    oninput={handleInput}
  />
  <button
    type="button"
    class="secret-reveal-button"
    aria-label={revealed ? `Hide ${label}` : `Reveal ${label}`}
    aria-pressed={revealed}
    onclick={() => (revealed = !revealed)}
  >
    <Icon name={revealed ? "eye-off" : "eye"} size={14} />
  </button>
</div>

<style>
  .secret-edit-input {
    position: relative;
    flex: 1;
    width: 100%;
    min-width: 0;
  }

  .control {
    padding-right: 34px;
  }

  .secret-reveal-button {
    position: absolute;
    top: 50%;
    right: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 25px;
    height: 25px;
    color: var(--text-muted);
    border-radius: var(--radius-sm);
    transform: translateY(-50%);
  }

  .secret-reveal-button:hover,
  .secret-reveal-button:focus-visible {
    color: var(--text-primary);
    background: var(--border);
  }

  .secret-reveal-button:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }
</style>
