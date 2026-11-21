import { setGlobalConfig } from 'valibot'

export default defineNuxtPlugin((nuxtApp) => {
  const i18n = (nuxtApp as any).$i18n
  const { locale } = i18n

  const configureValibot = (lang: string) => {
    // Sync Valibot with pt-BR whenever the active language is Portuguese
    if (lang.startsWith('pt')) {
      setGlobalConfig({ lang: 'pt-BR' })
    } else {
      setGlobalConfig({ lang })
    }
  }

  // Initialize
  configureValibot(locale.value)

  // Watch for changes
  watch(locale, (newLocale) => {
    configureValibot(newLocale)
  })
})
