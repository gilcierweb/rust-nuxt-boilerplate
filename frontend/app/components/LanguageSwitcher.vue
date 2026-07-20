<template>
  <div class="dropdown relative inline-flex [--auto-close:inside] [--offset:8] [--placement:bottom-end]">
    <button id="lang-dropdown-header" type="button" class="dropdown-toggle btn btn-text btn-circle size-10" aria-haspopup="menu" aria-expanded="false">
      <span class="icon-[tabler--language] size-5.5"></span>
    </button>
    <ul class="dropdown-menu dropdown-open:opacity-100 hidden min-w-32 shadow-xl border border-base-content/10 mt-2" role="menu">
      <li v-for="lang in locales" :key="lang.code">
        <button @click="switchLocale(lang.code)" class="dropdown-item flex items-center gap-2" :class="{ 'active': locale === lang.code }">
          <span class="text-lg">{{ (lang as any).flag }}</span>
          <span>{{ lang.name }}</span>
        </button>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
const { locale, locales, setLocale } = useI18n()
const localePath = useLocalePath()
const route = useRoute()

const STORAGE_KEY = 'preferredLanguage'

function setLanguagePreference(code: string) {
  localStorage.setItem(STORAGE_KEY, code)
}

async function switchLocale(code: string) {
  setLanguagePreference(code)
  await setLocale(code)
  await navigateTo(localePath(route.fullPath))
}

onMounted(() => {
  const saved = localStorage.getItem(STORAGE_KEY)
  if (saved && saved !== locale.value && locales.value.some((l: any) => l.code === saved)) {
    setLocale(saved)
  }
})
</script>
