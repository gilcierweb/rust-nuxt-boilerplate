<template>
  <nav class="flex items-center gap-2">
    <NuxtLink
        v-for="item in localeItems"
        :key="item.code"
        :to="switchLocalePath(item.code)"
        :class="[
        'rounded-md px-3 py-2 text-sm font-medium transition-colors',
        item.code === locale
          ? 'bg-primary text-white'
          : 'text-gray-500 hover:bg-gray-100 hover:text-gray-900'
      ]"
    >
      {{ item.name }}
    </NuxtLink>
  </nav>
</template>

<script setup lang="ts">
const { locale, locales } = useI18n()
const switchLocalePath = useSwitchLocalePath()

const localeItems = computed(() => {
  return locales.value.filter(
      (item): item is { code: string; name: string } =>
          typeof item !== 'string'
  )
})
</script>