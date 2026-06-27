<script lang="ts">
  import Icon from "../ui/Icon.svelte";
</script>

<div class="empty-detail">
  <Icon name="eye-off" size={26} />
  <p>Select an item to view its details</p>
</div>

<style>
  .empty-detail {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: var(--text-muted);
    gap: 12px;
    min-height: 200px;
    padding: 40px;
    text-align: center;
  }
</style>
