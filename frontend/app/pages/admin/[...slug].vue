<template>
  <div />
</template>

<script setup lang="ts">
const localePath = useLocalePath()
await navigateTo(localePath('/admin/dashboard'), { redirectCode: 302 })
</script>
