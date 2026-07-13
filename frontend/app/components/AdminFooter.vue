<template>
  <footer class="mx-auto w-full max-w-[1280px] px-6 py-3.5 text-sm">
    <div class="flex items-center justify-between gap-3 max-lg:flex-col">
      <p class="text-base-content text-center">
        &copy;{{ currentYear }}
        <span class="text-primary">{{ appName }}</span>
        , admin workspace for operations and telemetry oversight.
      </p>

      <div class="justify-enter flex items-center gap-4 max-sm:flex-col">
        <a href="/swagger-ui/" class="link link-primary link-animated font-normal" target="_blank">API Docs</a>
        <NuxtLink to="/contact" class="link link-primary link-animated font-normal">Contact</NuxtLink>
        <NuxtLink to="/privacy" class="link link-primary link-animated font-normal">Privacy</NuxtLink>
        <NuxtLink to="/terms" class="link link-primary link-animated font-normal">Terms</NuxtLink>
      </div>
    </div>
  </footer>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const appName = computed(() => runtimeConfig.public.appName || 'Buildtrack')
const currentYear = new Date().getFullYear()
</script>
