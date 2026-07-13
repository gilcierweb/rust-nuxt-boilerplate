<template>
  <div>
    <NuxtLoadingIndicator color="var(--color-primary)" />
    <NuxtRouteAnnouncer />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <AppToasts />
  </div>
</template>

<script setup lang="ts">
import AppToasts from '~/components/ui/AppToasts.vue'
</script>
