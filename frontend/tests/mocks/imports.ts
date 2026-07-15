export const useRoute = () => ({
  path: '/',
  params: {},
  query: {},
  fullPath: '/',
  matched: [],
})

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

export const useState = () => ref(null)

export const useRequestEvent = () => null
export const useRequestFetch = () => fetch

export const defineNuxtRouteMiddleware = (fn: any) => fn

export const ref = <T>(value: T) => ({ value } as any)
export const computed = (fn: any) => ({ value: fn() } as any)
export const reactive = (obj: any) => obj
