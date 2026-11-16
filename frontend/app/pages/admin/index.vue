<template>
  <div class="min-h-screen">
    <NuxtLoadingIndicator />
  </div>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'AdminIndexPage' })

definePageMeta({
  layout: 'admin'
})

const localePath = useLocalePath()
await navigateTo(localePath('/admin/dashboard'), { redirectCode: 302 })
</script>