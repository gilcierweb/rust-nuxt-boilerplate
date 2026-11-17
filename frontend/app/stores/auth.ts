import { defineStore } from 'pinia'

function isBackendUnavailable(err: unknown): boolean {
  const anyErr = err as any
  const status =
    anyErr?.status ?? anyErr?.statusCode ?? anyErr?.response?.status ?? anyErr?.data?.status
  if (status === 503) return true
  // ofetch's FetchError stores `data`/`status` as non-enumerable getters,
  // so JSON.stringify() omits them — check the direct properties first.
  const directCode =
    anyErr?.data?.error?.code ??
    anyErr?.response?._data?.error?.code ??
    anyErr?.error?.code
  if (directCode === 'BACKEND_UNAVAILABLE') return true
  if (anyErr?.status === 503 || anyErr?.statusCode === 503 || anyErr?.response?.status === 503) {
    return true
  }
  let serialized = ''
  try {
    serialized = JSON.stringify(anyErr?.data ?? anyErr?.response?._data ?? '')
  } catch {
    serialized = String(anyErr ?? '')
  }
  if (/BACKEND_UNAVAILABLE/i.test(serialized)) return true
  const message = String(anyErr?.message ?? anyErr?.data?.error?.message ?? '')
  return /backend (api )?unavailable/i.test(message)
}

import { csrfTokenFromCookieHeader } from '~~/shared/utils/csrf'
import type {
  User,
  AuthResponse,
  RefreshResponse,
  SessionResponse,
  LoginPayload,
  RegisterPayload,
} from '~/types'

// In-flight dedup promises. Client-only: the Nitro server runtime shares
// this module across concurrent SSR requests, so sharing a promise there
// would leak one request's tokens/state into another. On the server every
// action runs inline, scoped to its own Pinia store + request event.
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
  // Server: each SSR request initializes its own store instance, never the
  // module level (cross-request) init state.
  if (import.meta.server) return Promise.resolve()
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
  isWakingUp: boolean
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
    isWakingUp: false,
    returnUrl: null,
    isInitialized: false,
    isBootstrapping: false,
    hasSession: false,
  }),

  persist: false,

  getters: {
    isAuthenticated: (state) => !!state.accessToken && !!state.user,
    hasActiveSession: (state) => state.hasSession && !!state.user,
    isCreator: (state) => state.user?.roles?.includes('creator') ?? false,
    isAdmin: (state) =>
      state.user?.roles?.some((role) => role.toLowerCase() === 'admin') ?? false,
    isAgency: (state) => state.user?.roles?.includes('agency') ?? false,
    profileId: (state) => state.user?.profile_id ?? null,
    hasOtp: (state) => state.user?.is_otp_enabled ?? false,
    authHeader: (state) =>
      state.accessToken ? `Bearer ${state.accessToken}` : null,
  },

  actions: {
    async wakeBackend(timeoutMs = 180_000): Promise<boolean> {
      // Render free tier hibernates after 15 min of inactivity and takes
      // 30-60s to boot. A Cloudflare Worker cannot wake a sleeping Render
      // instance (Render's edge does not forward datacenter-origin requests
      // to the origin), so we wake it directly from the browser, whose
      // residential IP Render treats as a real user.
      this.isWakingUp = true
      const startedAt = Date.now()
      try {
        // 1. Try the Worker proxy first (fast path when warm).
        try {
          await $fetch('/health', { method: 'GET', credentials: 'omit' })
          return true
        } catch (err: any) {
          // Extract the backend origin URL from the 503 so we can call it
          // directly. Falls back to nothing (give up) if not provided.
          const backendUrl =
            err?.response?.headers?.get?.('x-backend-url') ??
            err?.data?.error?.backendUrl ??
            err?.response?._data?.error?.backendUrl ??
            err?.data?.backendUrl
          if (!backendUrl || typeof backendUrl !== 'string') {
            console.warn('[auth] wakeBackend: no backendUrl in 503 response', err)
            return false
          }

          // 2. Wake Render directly from the browser (residential IP).
          // Use no-cors mode: the response is opaque (can't read status), but
          // Render receives the request and wakes up. CORS is bypassed.
          const directHealthUrl = `${backendUrl.replace(/\/+$/, '')}/health`
          console.info('[auth] waking Render directly:', directHealthUrl)
          while (Date.now() - startedAt < timeoutMs) {
            try {
              await fetch(directHealthUrl, { method: 'GET', mode: 'no-cors' })
              console.info('[auth] wake request sent, confirming via Worker')
            } catch (directErr: any) {
              console.warn('[auth] direct wake attempt failed:', directErr?.message ?? directErr)
            }
            // 3. Confirm via the Worker proxy (Render should be waking).
            try {
              await $fetch('/health', { method: 'GET', credentials: 'omit' })
              console.info('[auth] Worker /health returned 200 — Render is awake')
              return true
            } catch {
              // Not ready yet. Wait and retry.
              const remaining = timeoutMs - (Date.now() - startedAt)
              if (remaining <= 0) return false
              await new Promise((resolve) =>
                setTimeout(resolve, Math.min(3_000, remaining)),
              )
            }
          }
          return false
        }
      } finally {
        this.isWakingUp = false
      }
    },

    async login(payload: LoginPayload) {
      const { $api } = useNuxtApp()
      this.isLoading = true
      try {
        // Wake the backend first when it is hibernating, so the login
        // request itself does not pay the cold-start penalty. Login is
        // attempted exactly once afterwards: it is not idempotent and must
        // not be retried in a loop (rate limit, lockout, audit logs).
        await this.wakeBackend()

        const data = await $api<AuthResponse>('/auth/login', {
          method: 'POST',
          body: payload,
        })
        this._setTokens(data)
        this.isInitialized = true
        return data
      } finally {
        this.isLoading = false
        this.isWakingUp = false
      }
    },

    async verifyMagicLink(token: string) {
      const { $api } = useNuxtApp()
      this.isLoading = true
      try {
        const data = await $api<AuthResponse>('/auth/magic-link/verify', {
          method: 'POST',
          body: { token },
        })
        this._setTokens(data)
        this.isInitialized = true
        return data
      } finally {
        this.isLoading = false
      }
    },

    async requestMagicLink(email: string) {
      const { $api } = useNuxtApp()
      this.isLoading = true
      try {
        await $api('/auth/magic-link', {
          method: 'POST',
          body: { email },
        })
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
      const localePath = useLocalePath()
      await navigateTo(localePath('/auth/login'))
    },

    async refreshTokens() {
      // Dedupe concurrent refreshes on the client only (see module note).
      if (import.meta.client && refreshPromise) return await refreshPromise

      const { $api } = useNuxtApp()
      const task = (async (): Promise<RefreshResponse> => {
        try {
          let data: RefreshResponse

          if (import.meta.server) {
            const event = useRequestEvent()
            const cookieHeader = event?.node?.req?.headers?.cookie
            const requestFetch = useRequestFetch()
            // The backend CSRF middleware requires the double-submit header
            // on the cookie-authenticated refresh POST (no Bearer is
            // attached on this branch).
            const csrfToken = csrfTokenFromCookieHeader(cookieHeader)
            data = await requestFetch<RefreshResponse>('/api/v1/auth/refresh', {
              method: 'POST',
              headers: {
                accept: 'application/json',
                ...(csrfToken ? { 'csrf-token': csrfToken } : {}),
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
          this._clear()
          this.isInitialized = true
          // Preserve the original error with status code
          const error = new Error('Session expired')
          ;(error as any).statusCode = err.statusCode || err.response?.status || 401
          throw error
        }
      })()

      if (!import.meta.client) return await task

      refreshPromise = task
      try {
        return await task
      } finally {
        refreshPromise = null
      }
    },

    async _bootstrapOnce() {
      this.isBootstrapping = true
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
      }
    },

    async bootstrapSession() {
      const needsClientHydration = import.meta.client && this.hasSession && !this.accessToken
      if (this.isInitialized && !needsClientHydration) return
      // Server: run inline, scoped to this request's store (see module note).
      if (!import.meta.client) {
        await this._bootstrapOnce()
        return
      }
      if (bootstrapPromise) return await bootstrapPromise

      // Create the promise FIRST, before awaiting, to prevent race conditions
      bootstrapPromise = this._bootstrapOnce()
      try {
        await bootstrapPromise
      } finally {
        bootstrapPromise = null
      }
    },

    // SSR version that receives pre-extracted context to avoid Nuxt context issues
    async fetchSessionSSR(
      event: { context?: Record<string, unknown> } | undefined,
      requestFetch: <T>(path: string, options?: Record<string, any>) => Promise<T>,
      cookieHeader: string | undefined,
    ) {
      const data = await requestFetch<SessionResponse>('/api/v1/auth/session', {
        method: 'GET',
        headers: {
          accept: 'application/json',
          ...(cookieHeader ? { cookie: cookieHeader } : {}),
        },
      })

      if (event?.context) {
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
      // The subject, its roles and its capabilities come from one place: the
      // auth me endpoint answers the effective grants in a flat payload, so
      // the store never rebuilds them from the token.
      const data = await $api<{
        id: string
        email: string
        roles: string[]
        permissions: string[]
        role: number
      }>('/auth/me')
      const user: User = {
        ...(this.user ?? {}),
        id: data.id,
        email: data.email,
        roles: data.roles ?? [],
        permissions: data.permissions ?? [],
        role: data.role ?? null,
      } as User
      this.user = user
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
