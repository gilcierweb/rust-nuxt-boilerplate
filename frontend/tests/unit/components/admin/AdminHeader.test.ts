import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed, ref } from 'vue'

// The header reads its dependencies as Nuxt auto-imports through the global
// scope. Real reactivity is required because the labels it exercises are
// computed from the auth store.
vi.unmock('vue')

import AdminHeader from '@/app/components/AdminHeader.vue'

const globalScope = globalThis as any
globalScope.computed = computed
globalScope.ref = ref
globalScope.useI18n = () => ({ t: (key: string) => key })
globalScope.useLocalePath = () => (path: string) => path
globalScope.useRuntimeConfig = () => ({ public: { appName: 'Test App' } })

let store: {
  user: { email: string; roles: string[] } | null
  logout: ReturnType<typeof vi.fn>
}

globalScope.useAuthStore = () => store

const mountHeader = () =>
  mount(AdminHeader, {
    global: {
      mocks: { $t: (key: string) => key },
      stubs: { NuxtLink: true, ThemeToggle: true, LanguageSwitcher: true },
    },
  })

describe('AdminHeader', () => {
  beforeEach(() => {
    store = { user: null, logout: vi.fn() }
  })

  it('renders the fallback initials without throwing when the user has no email', () => {
    const wrapper = mountHeader()

    // The i18n empty value is the fallback; its first two letters become the
    // initials. Before `t` was declared this computed threw a ReferenceError.
    expect(wrapper.text()).toContain('CO')
  })

  it('derives the initials from the email of a signed-in user', () => {
    store.user = { email: 'user06@example.com', roles: ['admin'] }

    const wrapper = mountHeader()

    expect(wrapper.text()).toContain('US')
  })

  it('marks FlyonUI-managed dropdown classes as hydration-mismatch-safe', () => {
    const wrapper = mountHeader()

    // FlyonUI mutates `open`/`hidden` before Vue hydrates. Both the dropdown
    // container and its menu must opt out of class mismatch warnings.
    const dropdownContainer = wrapper.find('#profile-dropdown').element.parentElement as HTMLElement
    const dropdownMenu = wrapper.find('ul[role="menu"]')

    expect(dropdownContainer.getAttribute('data-allow-mismatch')).toBe('class')
    expect(dropdownMenu.attributes('data-allow-mismatch')).toBe('class')
  })
})
