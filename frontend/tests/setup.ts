import '@testing-library/jest-dom/vitest'

// Nuxt auto-imports are not rewritten outside the Nuxt pipeline, so a component
// that calls useI18n() resolves the bare identifier from the global scope. The
// identity translation keeps the assertions on the i18n keys, not on sentences
// of a specific locale.
const globalScope = globalThis as any
globalScope.useI18n = () => ({ t: (key: string) => key })
globalScope.useLocalePath = () => (path: string) => path
// The Vue APIs a template compiler emits for slots and attributes.
globalScope.useSlots = (...args: any[]) => (globalThis as any).__vueUseSlots?.(...args)
globalScope.useAttrs = (...args: any[]) => (globalThis as any).__vueUseAttrs?.(...args)

import { vi } from 'vitest'
import { ref, computed, reactive, useSlots, useAttrs } from 'vue'

globalScope.__vueUseSlots = useSlots
globalScope.__vueUseAttrs = useAttrs

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

// A keyed singleton for the useState of the mocks below, the way Nuxt keeps
// one instance per key for the client: a composable that reads the same key
// twice shares the state and the initializer runs once per key. Both mocks
// share the one map so a key read through either import path is the same
// state.
const states = new Map<string, any>()
const useStateMock = <T,>(key: string, init: () => T) => {
  if (!states.has(key)) states.set(key, ref(init()))
  return states.get(key)
}

vi.mock('#imports', () => ({
  useI18n: () => ({ t: (key: string) => key }),
  useLocalePath: () => (path: string) => path,
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
  useState: useStateMock,
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
  useI18n: () => ({ t: (key: string) => key }),
  useLocalePath: () => (path: string) => path,
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
  useState: useStateMock,
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
