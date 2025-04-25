<template>
  <div v-html="typst.compiled" />
</template>

<script setup lang="ts">
import { reactive, onMounted, watch } from 'vue';
import { $typst } from '@myriaddreamin/typst.ts/contrib/snippet';

$typst.setCompilerInitOptions({
  beforeBuild: [],
  getModule: () =>
    'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-web-compiler/pkg/typst_ts_web_compiler_bg.wasm',
});

$typst.setRendererInitOptions({
  beforeBuild: [],
  getModule: () =>
    'https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
});

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
