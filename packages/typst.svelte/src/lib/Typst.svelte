<script lang="ts">
  import setTypst from './set-init-options-typst.ts';
  const { mainContent = '' }: { mainContent: string } = $props();
  const typst = setTypst();
</script>

{#await typst.svg({ mainContent }) then compiled}
  {@html compiled}
{/await}