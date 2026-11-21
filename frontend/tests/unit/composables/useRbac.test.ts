import { describe, expect, it, vi } from 'vitest'

vi.unmock('vue')

// The composable reads the session store: capabilities are the class-level
// codes the backend sends in `MeResponse.permissions` ("users:read",
// "roles:update", ...). A subject with no session reads as `[]`.
const globalScope = globalThis as any
globalScope.useAuthStore = () => authStore

// The store is read like Pinia state reads it: `authStore.user` is the
// unwrapped user object (`Ref<User | null>` in a real store), and the
// composable reads `.permissions` off it. Each test assigns a fresh object.
const authStore: { user: { permissions?: string[] } | null } = { user: null }

const { useRbac } = await import('~/composables/useRbac')

describe('useRbac', () => {
  it('reads the capability codes the subject holds', () => {
    authStore.user = { permissions: ['users:read', 'roles:update'] }
    const { permissions } = useRbac()
    expect(permissions.value).toEqual(['users:read', 'roles:update'])
  })

  it('answers can() by key, not by substring', () => {
    authStore.user = { permissions: ['roles:update'] }
    const { can } = useRbac()
    expect(can('roles:update')).toBe(true)
    expect(can('roles:delete')).toBe(false)
    expect(can('roles')).toBe(false)
    expect(can('roles:updated')).toBe(false)
  })

  it('a subject with no session can hold nothing', () => {
    authStore.user = null
    const { can, permissions } = useRbac()
    expect(can('anything:read')).toBe(false)
    expect(permissions.value).toEqual([])
  })
})
