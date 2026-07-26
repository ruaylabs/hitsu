<script lang="ts">
  import DetailFieldRow from "./DetailFieldRow.svelte";

  let {
    editFirstName = $bindable(""),
    editLastName = $bindable(""),
    editEmail = $bindable(""),
    editPhone = $bindable(""),
    editAddress = $bindable(""),
    editDob = $bindable(""),
  }: {
    editFirstName?: string;
    editLastName?: string;
    editEmail?: string;
    editPhone?: string;
    editAddress?: string;
    editDob?: string;
  } = $props();
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
    bind:value={editFirstName}
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
    bind:value={editLastName}
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
    bind:value={editEmail}
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
    bind:value={editPhone}
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
    bind:value={editAddress}
  />
</DetailFieldRow>
<DetailFieldRow label="Date of birth">
  <input
    class="control control--compact edit-input"
    type="date"
    aria-label="Date of birth"
    autocomplete="bday"
    bind:value={editDob}
  />
</DetailFieldRow>
