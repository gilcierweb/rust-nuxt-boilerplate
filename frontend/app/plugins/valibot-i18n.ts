import { setGlobalConfig } from 'valibot'

export default defineNuxtPlugin((nuxtApp) => {
  const i18n = (nuxtApp as any).$i18n
  const { locale } = i18n

  const configureValibot = (lang: string) => {
    // Sincroniza o Valibot sempre com pt-BR quando o idioma for português
    if (lang.startsWith('pt')) {
      setGlobalConfig({ lang: 'pt-BR' })
    } else {
      setGlobalConfig({ lang })
    }
  }

  // Inicializa
  configureValibot(locale.value)

  // Observa mudanças
  watch(locale, (newLocale) => {
    configureValibot(newLocale)
  })
})
