<template>
  <footer class="border-t border-base-content/10 bg-base-100 px-6 py-3">
    <p class="text-center text-xs text-base-content/60">
      {{ appName }} · Portal do Cliente
    </p>
  </footer>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const appName = computed(() => runtimeConfig.public.appName || 'Contifya')
</script>
