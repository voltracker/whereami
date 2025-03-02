<script setup lang="ts">
import Globe from "./components/Globe.vue";
</script>

<template>
  <ClientOnly>
    <Globe />
    <template #fallback> Loading map... </template>
  </ClientOnly>
</template>

<style></style>
