import { defineStore } from 'pinia'

import type {
  User,
  AuthResponse,
  RefreshResponse,
  SessionResponse,
  LoginPayload,
  RegisterPayload,
} from '~/types'

let bootstrapPromise: Promise<void> | null = null
let refreshPromise: Promise<RefreshResponse> | null = null
let initPromise: Promise<void> | null = null
let initResolve: (() => void) | null = null

// Create a promise that resolves when auth is initialized
function getInitPromise() {
  if (!initPromise) {
    initPromise = new Promise((resolve) => {
      initResolve = resolve
    })
  }
  return initPromise
}

// Call this when auth is initialized
function markInitialized() {
  if (initResolve) {
    initResolve()
    initResolve = null
  }
}

export function waitForAuthInit(timeout = 5000): Promise<void> {
  return Promise.race([
    getInitPromise(),
    new Promise<void>((_, reject) =>
      setTimeout(() => reject(new Error('Auth init timeout')), timeout)
    ),
  ])
}

interface AuthState {
  accessToken: string | null
  user: User | null
  isLoading: boolean
  returnUrl: string | null
  isInitialized: boolean
  isBootstrapping: boolean
  hasSession: boolean
}

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    accessToken: null,
    user: null,
    isLoading: false,
    returnUrl: null,
    isInitialized: false,
    isBootstrapping: false,
    hasSession: false,
  }),

  getters: {
    isAuthenticated: (state) => !!state.accessToken && !!state.user,
    hasActiveSession: (state) => state.hasSession && !!state.user,
    isCreator: (state) => state.user?.roles?.includes('creator') ?? false,
    isAdmin: (state) => state.user?.roles?.includes('admin') ?? false,
    isAgency: (state) => state.user?.roles?.includes('agency') ?? false,
    profileId: (state) => state.user?.profile_id ?? null,
    hasOtp: (state) => state.user?.is_otp_enabled ?? false,
    authHeader: (state) =>
      state.accessToken ? `Bearer ${state.accessToken}` : null,
  },

  actions: {
    async login(payload: LoginPayload) {
      const { $api } = useNuxtApp()
      this.isLoading = true
      try {
        const data = await $api<AuthResponse>('/auth/login', {
          method: 'POST',
          body: payload,
        })
        this._setTokens(data)
        this.isInitialized = true
        return data
      } finally {
        this.isLoading = false
      }
    },

    async register(payload: RegisterPayload) {
      const { $api } = useNuxtApp()
      this.isLoading = true
      try {
        return await $api('/auth/register', {
          method: 'POST',
          body: payload,
        })
      } finally {
        this.isLoading = false
      }
    },

    async logout() {
      const { $api } = useNuxtApp()
      await $api('/auth/logout', {
        method: 'POST',
      }).catch(() => {}) // ignore logout errors
      this._clear()
      await navigateTo('/auth/login')
    },

    async refreshTokens() {
      if (refreshPromise) return await refreshPromise

      const { $api } = useNuxtApp()
      refreshPromise = (async () => {
        try {
          let data: RefreshResponse

          if (import.meta.server) {
            const event = useRequestEvent()
            const cookieHeader = event?.node?.req?.headers?.cookie
            const requestFetch = useRequestFetch()
            data = await requestFetch<RefreshResponse>('/api/v1/auth/refresh', {
              method: 'POST',
              headers: {
                accept: 'application/json',
                ...(cookieHeader ? { cookie: cookieHeader } : {}),
              },
            })
          } else {
            data = await $api<RefreshResponse>('/auth/refresh', {
              method: 'POST',
            })
          }

          if (import.meta.server) {
            const event = useRequestEvent()
            if (event) {
              event.context.authAccessToken = data.access_token
            }
          }

          this.accessToken = data.access_token
          this.hasSession = true
          this.isInitialized = true
          return data
        } catch (err: any) {
          // eslint-disable-next-line no-console
          console.log('[AuthStore] refreshTokens - error:', err?.statusCode || err?.response?.status, err?.message)
          this._clear()
          this.isInitialized = true
          // Preserve the original error with status code
          const error = new Error('Session expired')
          ;(error as any).statusCode = err.statusCode || err.response?.status || 401
          throw error
        } finally {
          refreshPromise = null
        }
      })()

      return await refreshPromise
    },

    async bootstrapSession() {
      const needsClientHydration = import.meta.client && this.hasSession && !this.accessToken
      if (this.isInitialized && !needsClientHydration) return
      if (bootstrapPromise) return await bootstrapPromise

      this.isBootstrapping = true

      bootstrapPromise = (async () => {
        try {
          await this.refreshTokens()
          if (!this.user) {
            await this.fetchMe()
          }
        } catch {
          this._clear()
          this.isInitialized = true
        } finally {
          this.isBootstrapping = false
          bootstrapPromise = null
        }
      })()

      await bootstrapPromise
    },

    // SSR version that receives pre-extracted context to avoid Nuxt context issues
    async fetchSessionSSR(event: any, requestFetch: any, cookieHeader: string | undefined) {
      const data = await requestFetch<SessionResponse>('/api/v1/auth/session', {
        method: 'GET',
        headers: {
          accept: 'application/json',
          ...(cookieHeader ? { cookie: cookieHeader } : {}),
        },
      })

      if (event) {
        event.context.authAccessToken = data.access_token
      }

      this.accessToken = data.access_token
      this._setSessionUser(data.user)
      this.hasSession = true
      this.isInitialized = true
      return data
    },

    async fetchSession() {
      let data: SessionResponse

      if (import.meta.server) {
        // SSR: use useRequestEvent and useRequestFetch - must be called within Nuxt context
        const event = useRequestEvent()
        const cookieHeader = event?.node?.req?.headers?.cookie
        const requestFetch = useRequestFetch()
        data = await requestFetch<SessionResponse>('/api/v1/auth/session', {
          method: 'GET',
          headers: {
            accept: 'application/json',
            ...(cookieHeader ? { cookie: cookieHeader } : {}),
          },
        })

        if (event) {
          event.context.authAccessToken = data.access_token
        }
      } else {
        // Client: same-origin proxy endpoint with cookie credentials.
        data = await $fetch<SessionResponse>('/api/v1/auth/session', {
          method: 'GET',
          credentials: 'include',
          headers: {
            accept: 'application/json',
          },
        })
      }

      this.accessToken = data.access_token
      this._setSessionUser(data.user)
      this.hasSession = true
      this.isInitialized = true
      markInitialized()
      return data
    },

    async fetchMe() {
      const { $api } = useNuxtApp()
      const data = await $api<{ user: User; profile: any; roles: string[] }>('/users/me')
      if (this.user) {
        this.user = { ...this.user, ...data.user, roles: data.roles }
      } else {
        this.user = { ...data.user, roles: data.roles }
      }
      this.hasSession = true
      this.isInitialized = true
      markInitialized()
    },

    _setTokens(data: AuthResponse) {
      this.accessToken = data.access_token
      this._setSessionUser(data.user)
      this.hasSession = true
    },

    _setSessionUser(user: User) {
      this.user = {
        id: user.id,
        email: user.email,
        profile_id: user.profile_id,
        roles: user.roles,
        is_otp_enabled: user.is_otp_enabled,
      }
    },

    _clear() {
      this.accessToken = null
      this.user = null
      this.isBootstrapping = false
      this.hasSession = false
    },

    setReturnUrl(url: string) {
      this.returnUrl = url
    },
  },
})
