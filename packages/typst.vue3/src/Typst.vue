<template>
  <div ref="typst" />
</template>

<script setup lang="ts">
import { ref, defineProps, onMounted, watch } from 'vue';
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';

interface prop {
  content: string;
}

const typst = ref(null)

const props = withDefaults(defineProps<prop>(), {
  content: ''
});

onMounted(async () => {
  typst.value =  await $typst.svg({mainContent: props.content})
})

watch(props, async(newVal, oldVal) => {
  typst.value =  await $typst.svg({mainContent: newVal})
})

</script>

<style lang="css"></style>
