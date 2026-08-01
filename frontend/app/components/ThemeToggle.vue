<template>
  <label class="swap swap-rotate">
    <input
      type="checkbox"
      :value="DARK_THEME"
      class="theme-controller"
      :checked="isDark"
      :aria-label="$t('admin.theme.toggle')"
      @change="onToggle"
    />
    <span class="swap-off icon-[tabler--sun] size-7" aria-hidden="true"></span>
    <span class="swap-on icon-[tabler--moon] size-7" aria-hidden="true"></span>
  </label>
</template>

<script setup lang="ts">
const DARK_THEME = 'dark'
const DEFAULT_THEME = 'corporate'
const COOKIE_NAME = 'admin-theme'

const themeCookie = useCookie<string>(COOKIE_NAME, {
  default: () => DEFAULT_THEME,
  sameSite: 'lax',
  maxAge: 60 * 60 * 24 * 365,
})

const isDark = computed(() => themeCookie.value === DARK_THEME)

useHead({
  htmlAttrs: {
    'data-theme': themeCookie,
  },
})

function onToggle(event: Event) {
  const checked = (event.target as HTMLInputElement).checked
  themeCookie.value = checked ? DARK_THEME : DEFAULT_THEME
}
</script>
