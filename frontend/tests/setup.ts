import '@testing-library/jest-dom/vitest'

// Nuxt auto-imports are not rewritten outside the Nuxt pipeline, so a component
// that calls useI18n() resolves the bare identifier from the global scope. The
// identity translation keeps the assertions on the i18n keys, not on sentences
// of a specific locale.
const globalScope = globalThis as any

// Nuxt `useState` auto-import: keyed singletons backed by the real ref, the
// way the client behaves. Tests that model the server/client distinction use
// a fresh map themselves (see `useToast.spec`).
import { ref } from 'vue'
const useStateStates = new Map<string, any>()
globalScope.useState = <T,>(key: string, init: () => T) => {
  if (!useStateStates.has(key)) useStateStates.set(key, ref(init()))
  return useStateStates.get(key)
}
globalScope.useI18n = () => ({ t: (key: string) => key })
globalScope.useLocalePath = () => (path: string) => path
// The Vue APIs a template compiler emits for slots and attributes.
globalScope.useSlots = (...args: any[]) => (globalThis as any).__vueUseSlots?.(...args)
globalScope.useAttrs = (...args: any[]) => (globalThis as any).__vueUseAttrs?.(...args)

// VueUse composables are auto-imported by @vueuse/nuxt outside the Nuxt
// pipeline too. The mock adds and removes the real listener on the resolved
// element, so a component that binds Escape exercises the wiring it ships.
globalScope.useEventListener = (
  target: any,
  event: string,
  handler: (event: any) => void,
) => {
  const element = typeof target === 'function' ? target() : target
  element?.addEventListener?.(event, handler)
  return () => element?.removeEventListener?.(event, handler)
}

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

// Vue is NOT mocked here. The previous shim replaced `ref`, `computed` and
// `watch` with non reactive stand-ins, so every watcher in a mounted
// component was a no-op and the suite reported green while none of the
// reactive contracts (a form seeded by the row that answered last, a
// selection cleared on page change) was exercised. The real reactivity runs
// everywhere; a test that needs a Nuxt auto-import resolves it from the
// global scope the way the app does.

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
