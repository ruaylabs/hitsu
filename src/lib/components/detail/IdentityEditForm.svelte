<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";
  import type { EditFormState } from "./editForm";

  let { form }: { form: EditFormState } = $props();
</script>

<DetailFieldRow label="First name">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="First name"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.firstName}
  />
</DetailFieldRow>
<DetailFieldRow label="Last name">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Last name"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.lastName}
  />
</DetailFieldRow>
<DetailFieldRow label="Email">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Email"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.email}
  />
</DetailFieldRow>
<DetailFieldRow label="Phone">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Phone"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.phone}
  />
</DetailFieldRow>
<DetailFieldRow label="Address">
  <input
    class="control control--compact edit-input"
    type="text"
    placeholder="Address"
    autocomplete="off"
    autocorrect="off"
    autocapitalize="off"
    spellcheck="false"
    bind:value={form.address}
  />
</DetailFieldRow>
<DetailFieldRow label="Date of birth">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Date of birth"
    autocomplete="bday"
    bind:value={form.dob}
  />
</DetailFieldRow>
