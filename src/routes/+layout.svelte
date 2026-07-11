<script lang="ts">
  import "@tabler/icons-webfont/dist/tabler-icons.css";
  import "../lib/styles/tokens.css";
  import "../lib/styles/reset.css";
  import "../lib/styles/global.css";
  import "../lib/styles/controls.css";
  import ToastStack from "$lib/components/ui/ToastStack.svelte";
</script>

<slot />
<ToastStack />
