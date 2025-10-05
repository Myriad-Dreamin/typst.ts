<template>
  <div v-html="typst.compiled" />
</template>

<script setup lang="ts">
import { reactive, onMounted, watch } from 'vue';
import { $typst } from '@myriaddreamin/typst.ts';
import setTypst from './set-init-options-typst.ts';

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
  setTypst();
  typst.compiled = await $typst.svg({ mainContent: props.content });
});

watch(()=>props.content, async (newVal, _) => {
  typst.compiled = await $typst.svg({ mainContent: newVal });
});
</script>

<style lang="css"></style>
