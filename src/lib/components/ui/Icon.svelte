<script lang="ts">
  let {
    name,
    size = 15,
    label,
    style,
    ...rest
  }: {
    name: string;
    size?: number;
    label?: string;
    style?: string;
  } = $props();

  let aria = $derived(label ? { "aria-label": label } : { "aria-hidden": "true" as const });
</script>

<i class="ti ti-{name}" style="font-size: {size}px; {style ?? ''}" {...aria} {...rest}></i>
