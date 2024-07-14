<script setup lang="ts">
import Typst from '../src/Typst.vue';

const sourceCode = '= Hello World!\nThis is a typst document compiled in browser.';
</script>

<template>
  <Typst v-bind:content="sourceCode" />
</template>

<style lang="css"></style>
