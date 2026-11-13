export const useRoute = () => ({
  path: '/',
  params: {},
  query: {},
  fullPath: '/',
  matched: [],
})

// The tests assert on the i18n keys, not on the translated sentences: the
// identity translation keeps every assertion independent of the locale files.
export const useI18n = () => ({ t: (key: string) => key })
export const useLocalePath = () => (path: string) => path

export const navigateTo = () => {}

export const useRuntimeConfig = () => ({
  public: {
    apiBase: '/api/v1',
    wsBase: 'ws://localhost:8080/api/v1',
    cdnUrl: 'https://cdn.example.com',
    stripeKey: '',
    appName: 'Test App',
    apiDirectBase: '',
  },
  backendApiBase: 'http://localhost:8080/api/v1',
  backendApiKey: '',
})

export const useNuxtApp = () => ({
  $pinia: {},
  $api: () => Promise.resolve(),
})

// A keyed singleton within the test process, the way Nuxt keeps one instance
// per key for the client, so a composable that reads the same key twice
// shares the state and the initializer runs once per key.
const states = new Map<string, { value: unknown }>()
export const useState = <T,>(key: string, init: () => T) => {
  if (!states.has(key)) states.set(key, ref(init()))
  return states.get(key) as { value: T }
}

export const useRequestEvent = () => null
export const useRequestFetch = () => fetch

export const defineNuxtRouteMiddleware = (fn: any) => fn

export const ref = <T>(value: T) => ({ value } as any)
export const computed = (fn: any) => ({ value: fn() } as any)
export const reactive = (obj: any) => obj
