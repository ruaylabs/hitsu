<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import DetailFields from "./DetailFields.svelte";

  let { entry }: { entry: Entry } = $props();

  const groups = $derived.by(() => {
    const p = entry.passport;
    if (!p) return [];
    return [
      [
        { label: "Type", value: p.type },
        { label: "Issuing country", value: p.issuingCountry },
        {
          label: "Number",
          secret: p.hasNumber ? { field: "passportNumber" as const } : undefined,
        },
        { label: "Full name", value: p.fullName },
        { label: "Sex", value: p.sex },
        { label: "Nationality", value: p.nationality },
        { label: "Issuing authority", value: p.issuingAuthority },
      ],
      [
        { label: "Date of birth", value: p.birthDate },
        { label: "Place of birth", value: p.birthPlace },
        { label: "Issued on", value: p.issueDate },
        { label: "Expiry date", value: p.expiryDate },
      ],
    ];
  });
</script>

<DetailFields entryId={entry.id} {groups} />
