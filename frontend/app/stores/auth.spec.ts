import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useAuthStore } from '@/app/stores/auth'

describe('Auth Store', () => {
  beforeEach(() => {
    // Creates a fresh pinia and makes it active
    // so it's automatically used by any useStore() call
    // without having to pass it to it: `useStore(pinia)`
    const pinia = createPinia()
    setActivePinia(pinia)
  })

  it('initializes with default state', () => {
    const authStore = useAuthStore()
    expect(authStore.user).toBeNull()
    expect(authStore.accessToken).toBeNull()
    expect(authStore.refreshToken).toBeNull()
    expect(authStore.isAuthenticated).toBe(false)
  })

  it('sets user and tokens on login', () => {
    const authStore = useAuthStore()
    const user = { id: '1', email: 'test@example.com' }
    const accessToken = 'access-token'
    const refreshToken = 'refresh-token'
    
    authStore.login(user, accessToken, refreshToken)
    
    expect(authStore.user).toEqual(user)
    expect(authStore.accessToken).toEqual(accessToken)
    expect(authStore.refreshToken).toEqual(refreshToken)
    expect(authStore.isAuthenticated).toBe(true)
  })

  it('clears user and tokens on logout', () => {
    const authStore = useAuthStore()
    // First login
    authStore.login({ id: '1', email: 'test@example.com' }, 'access-token', 'refresh-token')
    
    // Then logout
    authStore.logout()
    
    expect(authStore.user).toBeNull()
    expect(authStore.accessToken).toBeNull()
    expect(authStore.refreshToken).toBeNull()
    expect(authStore.isAuthenticated).toBe(false)
  })
})