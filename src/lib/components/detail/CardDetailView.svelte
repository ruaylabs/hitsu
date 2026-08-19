<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import { cardBrandName, formatCardNumber } from "$lib/utils/format";
  import DetailFields from "./DetailFields.svelte";

  let { entry }: { entry: Entry } = $props();

  // Masked by default like the other secrets: the full PAN only crosses IPC
  // on an explicit reveal, never on selection.
  const groups = $derived.by(() => {
    const c = entry.card;
    if (!c) return [];
    return [
      [
        { label: "Type", value: c.type ? cardBrandName(c.type) : undefined },
        { label: "Holder", value: c.holder },
        {
          label: "Number",
          secret: c.hasNumber
            ? {
                field: "cardNumber" as const,
                masked: c.numberMasked,
                format: (value: string) => formatCardNumber(value, c.type),
              }
            : undefined,
        },
        {
          label: "Expires",
          value:
            c.expMonth && c.expYear
              ? `${String(c.expMonth).padStart(2, "0")}/${c.expYear}`
              : undefined,
        },
        { label: "CVV", secret: c.hasCvv ? { field: "cardCvv" as const } : undefined },
        { label: "PIN", secret: c.hasPin ? { field: "cardPin" as const } : undefined },
      ],
    ];
  });
</script>

<DetailFields entryId={entry.id} {groups} />
