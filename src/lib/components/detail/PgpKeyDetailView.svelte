<script lang="ts">
  import type { Entry } from "$lib/bridge/types";
  import DetailFields from "./DetailFields.svelte";

  let { entry }: { entry: Entry } = $props();

  const groups = $derived.by(() => {
    const pgp = entry.pgpKey;
    if (!pgp) return [];
    return [
      [
        { label: "Fingerprint", value: pgp.fingerprint, mono: true, copy: true },
        { label: "Key ID", value: pgp.keyId, mono: true, copy: true },
        { label: "User IDs", value: pgp.userIds },
        { label: "Algorithm", value: pgp.algorithm },
        { label: "Expires", value: pgp.expiresAt },
      ],
      [{ label: "Public key", value: pgp.publicKey, mono: true, copy: true }],
      [
        {
          label: "Private key",
          secret: pgp.hasPrivateKey ? { field: "pgpPrivateKey" as const } : undefined,
        },
      ],
    ];
  });
</script>

<DetailFields entryId={entry.id} {groups} />
