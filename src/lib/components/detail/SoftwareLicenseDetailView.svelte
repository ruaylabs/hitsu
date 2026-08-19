<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import DetailFields from "./DetailFields.svelte";

  let { entry }: { entry: Entry } = $props();

  const groups = $derived.by(() => {
    const l = entry.softwareLicense;
    if (!l) return [];
    return [
      [
        { label: "Version", value: l.version },
        {
          label: "License key",
          secret: l.hasLicenseKey ? { field: "licenseKey" as const } : undefined,
        },
      ],
      [
        { label: "Licensed to", value: l.licensedTo },
        { label: "Registered email", value: l.registeredEmail, copy: true },
        { label: "Company", value: l.company },
      ],
      [
        { label: "Download page", value: l.downloadPage, copy: true, link: true },
        { label: "Publisher", value: l.publisher },
        { label: "Website", value: l.website, copy: true, link: true },
        { label: "Retail price", value: l.retailPrice },
        { label: "Support email", value: l.supportEmail, copy: true },
      ],
      [
        { label: "Purchase date", value: l.purchaseDate },
        { label: "Order number", value: l.orderNumber },
        { label: "Order total", value: l.orderTotal },
      ],
    ];
  });
</script>

<DetailFields entryId={entry.id} {groups} />
