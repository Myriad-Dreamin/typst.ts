<script lang="ts">
  import { $typst as typstM } from '@myriaddreamin/typst.ts';
  import setTypst from './set-init-options-typst.ts';
  import { onMount } from 'svelte';

  let compiled = $state('');
  const { mainContent = '' }: { mainContent: string } = $props();

  onMount(async () => {
    setTypst();
    compiled = await typstM.svg({ mainContent });
  });

  $effect(() => {
    typstM.svg({ mainContent }).then(newCompiled => {
      compiled = newCompiled;
    });
  });
</script>

{@html compiled}
