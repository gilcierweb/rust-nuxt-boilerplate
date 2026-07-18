import '@testing-library/jest-dom/vitest'
import { vi } from 'vitest'
import { ref, computed, reactive } from 'vue'

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

class MockResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

Object.defineProperty(globalThis, 'ResizeObserver', {
  writable: true,
  value: MockResizeObserver,
})

class MockIntersectionObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

Object.defineProperty(globalThis, 'IntersectionObserver', {
  writable: true,
  value: MockIntersectionObserver,
})

vi.mock('#imports', () => ({
  useRoute: () => ({
    path: '/',
    params: {},
    query: {},
    fullPath: '/',
    matched: [],
  }),
  navigateTo: () => {},
  useRuntimeConfig: () => ({
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
  }),
  useNuxtApp: () => ({
    $pinia: {},
    $api: () => Promise.resolve({ data: 'test' }),
    $fetch: () => Promise.resolve({}),
  }),
  useState: () => ref(null),
  useRequestEvent: () => null,
  useRequestFetch: () => Promise.resolve({}),
  defineNuxtRouteMiddleware: (fn: any) => fn,
  ref: <T>(value: T) => ref(value),
  computed: (fn: any) => computed(fn),
  reactive: (obj: any) => reactive(obj),
  useFormAlert: () => ({
    setFormAlertMessage: vi.fn(),
    clearFormAlertMessage: vi.fn(),
  }),
}))

vi.mock('pinia', () => ({
  createPinia: () => ({
    install: () => {},
  }),
  setActivePinia: () => {},
  defineStore: () => () => ({
    $state: {},
    $reset: () => {},
  }),
}))

vi.mock('vue', async () => {
  const actual = await vi.importActual('vue')
  return {
    ...actual,
    ref: <T>(value: T) => ({ value } as any),
    computed: (fn: any) => ({ value: fn() } as any),
    reactive: (obj: any) => obj,
    onMounted: (fn: any) => fn(),
    watch: () => {},
  }
})

const mockDefinePageMeta = () => {}
vi.mock('nuxt/app', () => ({
  useRoute: () => ({
    path: '/',
    params: {},
    query: {},
    fullPath: '/',
    matched: [],
  }),
  navigateTo: () => {},
  useRuntimeConfig: () => ({
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
  }),
  useNuxtApp: () => ({
    $pinia: {},
    $api: () => Promise.resolve({ data: 'test' }),
    $fetch: () => Promise.resolve({}),
  }),
  useState: () => ref(null),
  useRequestEvent: () => null,
  useRequestFetch: () => Promise.resolve({}),
  defineNuxtRouteMiddleware: (fn: any) => fn,
  definePageMeta: mockDefinePageMeta,
  ref: <T>(value: T) => ref(value),
  computed: (fn: any) => computed(fn),
  reactive: (obj: any) => reactive(obj),
  useFormAlert: () => ({
    setFormAlertMessage: vi.fn(),
    clearFormAlertMessage: vi.fn(),
  }),
}))
