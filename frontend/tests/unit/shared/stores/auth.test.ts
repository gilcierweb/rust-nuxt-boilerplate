import { describe, expect, it, vi } from 'vitest'

// The shared setup replaces `vue` and `pinia` with shims for the utility
// tests. A store test needs the real reactive state and the real store.
vi.unmock('vue')
vi.unmock('pinia')

import { createPinia, setActivePinia } from 'pinia'

const meCalls: string[] = []

// The payload of GET /auth/me answers flat: the subject, its roles, its
// capabilities and the presentation role code. There is no nested user key.
const mePayload = {
  id: 'user-1',
  email: 'admin@example.com',
  roles: ['ADMIN'],
  permissions: ['users:read', 'roles:update'],
  role: 1,
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

  it('carries the identity, the roles and the capability list of the flat payload', async () => {
    const store = freshStore()

    await store.fetchMe()

    expect(store.user?.id).toBe('user-1')
    expect(store.user?.email).toBe('admin@example.com')
    expect(store.user?.roles).toEqual(['ADMIN'])
    expect(store.user?.permissions).toEqual(['users:read', 'roles:update'])
    expect(store.user?.role).toBe(1)
  })

  it('answers the admin check whatever the case of the role name', () => {
    const store = freshStore()

    store.user = {
      id: 'user-1',
      email: 'admin@example.com',
      profile_id: 'profile-1',
      roles: ['ADMIN'],
      is_otp_enabled: false,
    }
    expect(store.isAdmin).toBe(true)

    store.user = {
      id: 'user-1',
      email: 'admin@example.com',
      profile_id: 'profile-1',
      roles: ['admin'],
      is_otp_enabled: false,
    }
    expect(store.isAdmin).toBe(true)

    store.user = {
      id: 'user-1',
      email: 'admin@example.com',
      profile_id: 'profile-1',
      roles: ['manager'],
      is_otp_enabled: false,
    }
    expect(store.isAdmin).toBe(false)
  })
})
