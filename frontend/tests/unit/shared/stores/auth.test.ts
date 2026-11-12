import { describe, expect, it, vi } from 'vitest'

// The shared setup replaces `vue` and `pinia` with shims for the utility
// tests. A store test needs the real reactive state and the real store.
vi.unmock('vue')
vi.unmock('pinia')

import { createPinia, setActivePinia } from 'pinia'

const meCalls: string[] = []

const mePayload = {
  user: {
    id: 'user-1',
    email: 'user01@example.com',
    profile_id: null,
    roles: ['ADMIN'],
    is_otp_enabled: false,
  },
  profile: null,
  roles: ['ADMIN'],
  permissions: ['users:read', 'roles:update'],
  role: 'ADMIN',
}

// The auto-imports the store resolves from the global scope, the same shape
// the shared setup uses for the components.
const globalScope = globalThis as any
globalScope.useNuxtApp = () => ({
  $api: async (endpoint: string) => {
    meCalls.push(endpoint)
    if (endpoint === '/auth/me') return mePayload
    return {}
  },
})
globalScope.useLocalePath = () => (path: string) => path
globalScope.navigateTo = async () => {}

import { useAuthStore } from '@/app/stores/auth'

function freshStore() {
  setActivePinia(createPinia())
  return useAuthStore()
}

describe('auth store capabilities', () => {
  it('reads the subject from the auth me endpoint, not the dead users me', async () => {
    const store = freshStore()
    meCalls.length = 0

    await store.fetchMe()

    expect(meCalls).toEqual(['/auth/me'])
  })

  it('carries the roles and the capability list of the payload', async () => {
    const store = freshStore()

    await store.fetchMe()

    expect(store.user?.roles).toEqual(['ADMIN'])
    expect(store.user?.permissions).toEqual(['users:read', 'roles:update'])
  })

  it('answers the admin check whatever the case of the role name', () => {
    const store = freshStore()

    store.user = { ...mePayload.user, roles: ['ADMIN'] }
    expect(store.isAdmin).toBe(true)

    store.user = { ...mePayload.user, roles: ['admin'] }
    expect(store.isAdmin).toBe(true)

    store.user = { ...mePayload.user, roles: ['manager'] }
    expect(store.isAdmin).toBe(false)
  })
})
