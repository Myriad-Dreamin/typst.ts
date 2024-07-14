<template>
  <div v-html="typst.compiled" />
</template>

<script setup lang="ts">
import { reactive, onMounted, watch } from 'vue';
import { $typst } from '@myriaddreamin/typst.ts/dist/esm/contrib/snippet.mjs';

interface prop {
  content: string;
}

const typst = reactive({
  compiled: '',
});

const props = withDefaults(defineProps<prop>(), {
  content: '',
});

onMounted(async () => {
  typst.compiled = await $typst.svg({ mainContent: props.content });
});

watch(props, async (newVal, oldVal) => {
  typst.compiled = await $typst.svg({ mainContent: newVal });
});
</script>

<style lang="css"></style>
