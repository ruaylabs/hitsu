<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import DetailFields from "./DetailFields.svelte";

  let { entry }: { entry: Entry } = $props();

  const groups = $derived.by(() => {
    const i = entry.identity;
    if (!i) return [];
    return [
      [
        { label: "First name", value: i.firstName },
        { label: "Last name", value: i.lastName },
        { label: "Email", value: i.email, copy: true },
        { label: "Phone", value: i.phone, copy: true },
        { label: "Address", value: i.address },
        { label: "Date of birth", value: i.dob },
      ],
    ];
  });
</script>

<DetailFields entryId={entry.id} {groups} />
