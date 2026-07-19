<template>
  <div class="min-h-screen">
    <NuxtLoadingIndicator />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'admin'
})

const localePath = useLocalePath()
await navigateTo(localePath('/admin/dashboard'), { redirectCode: 302 })
</script>