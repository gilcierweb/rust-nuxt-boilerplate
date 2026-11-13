import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import { computed, ref } from 'vue'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A mounted menu needs the real reactivity of its computed labels.
vi.unmock('vue')

import PortalNavbar from '@/app/components/PortalNavbar.vue'

// The navbar reads its dependencies as Nuxt auto-imports, through the global
// scope: the spec installs the real reactivity, a controllable auth store,
// the runtime config and the keyed useState singleton. The translation
// answers the key, and the existence check knows the six roles of the
// catalog, so a missing key is exercised on purpose instead of being the
// fate of every key.
const globalScope = globalThis as any
globalScope.computed = computed
globalScope.ref = ref

const CATALOG_ROLES = ['admin', 'manager', 'editor', 'viewer', 'support', 'customer']
globalScope.useI18n = () => ({
  t: (key: string) => key,
  te: (key: string) => {
    const match = /^portal\.navbar\.roles\.(.+)$/.exec(key)
    return match ? CATALOG_ROLES.includes(match[1]) : true
  },
})

const states = new Map<string, { value: unknown }>()
globalScope.useState = <T,>(key: string, init: () => T) => {
  if (!states.has(key)) states.set(key, { value: init() })
  return states.get(key) as { value: T }
}

let store: {
  user: { email: string; roles: string[] } | null
  logout: ReturnType<typeof vi.fn>
}

globalScope.useAuthStore = () => store
globalScope.useRuntimeConfig = () => ({ public: { appName: 'Test App' } })

const mountNavbar = () =>
  mount(PortalNavbar, {
    global: {
      // The template reads $t, the same identity translation the shared setup
      // gives the composables: the assertions stay on the keys.
      mocks: {
        $t: (key: string) => key,
      },
      stubs: {
        NuxtLink: true,
        ThemeToggle: true,
        LanguageSwitcher: true,
      },
    },
  })

describe('PortalNavbar', () => {
  beforeEach(() => {
    states.clear()
    store = {
      user: { email: 'user06@example.com', roles: ['customer'] },
      logout: vi.fn(),
    }
  })

  it('names the subject and its role in the language of the viewer', () => {
    const wrapper = mountNavbar()

    // The identity translation answers the key, which is the contract: the
    // label travels through i18n instead of a hardcoded sentence.
    expect(wrapper.text()).toContain('user06@example.com')
    expect(wrapper.text()).toContain('portal.navbar.roles.customer')
  })

  it('keeps the slug of a role the locale does not know', () => {
    store = { user: { email: 'a@b.co', roles: ['auditor'] }, logout: vi.fn() }
    const wrapper = mountNavbar()

    expect(wrapper.text()).toContain('auditor')
  })

  it('joins every role the subject holds', () => {
    store = { user: { email: 'a@b.co', roles: ['manager', 'editor'] }, logout: vi.fn() }
    const wrapper = mountNavbar()

    expect(wrapper.text()).toContain(
      'portal.navbar.roles.manager / portal.navbar.roles.editor',
    )
  })

  it('leaves the role line out when the subject holds none', () => {
    store = { user: { email: 'a@b.co', roles: [] }, logout: vi.fn() }
    const wrapper = mountNavbar()

    expect(wrapper.text()).not.toContain('portal.navbar.roles.')
  })

  it('names its toggle for assistive technology', () => {
    const wrapper = mountNavbar()

    expect(wrapper.find('button#portal-profile-dropdown').attributes('aria-label')).toBe(
      'portal.navbar.userMenu',
    )
  })

  it('signs out through the store', async () => {
    const wrapper = mountNavbar()

    await wrapper.find('button.btn-error').trigger('click')

    expect(store.logout).toHaveBeenCalledOnce()
  })

  it('offers only destinations the portal really has', () => {
    const wrapper = mountNavbar()

    expect(wrapper.findAll('a')).toHaveLength(0) // NuxtLink is stubbed
    expect(wrapper.findAllComponents({ name: 'NuxtLink' })).toHaveLength(3)
  })
})
