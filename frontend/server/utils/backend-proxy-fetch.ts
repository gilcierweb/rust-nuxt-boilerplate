import {
  setResponseHeader,
  setResponseStatus,
  type H3Event,
} from 'h3'

import {
  BackendHealthProbeExhaustedError,
  createBackendWakeupState,
  isBackendHealthProbeExhaustedError,
  isRenderWakeupError,
  waitForBackendHealth,
  type BackendWakeupState,
} from '~~/server/utils/backend-wakeup'

export const BACKEND_REQUEST_MAX_ATTEMPTS = 3

// The wake-up path is generic HTTP: a cold or hibernating host answers a
// 429/502/503/504 (or drops the connection), the caller probes readiness
// and retries. The knobs below only shape that path and are all optional:
// a warm backend on a VPS or a cloud VM never takes it, and any value here
// is a plain HTTP header or a ceiling, inert to a host that is always up.

function readEnvNumber(name: string, fallback: number): number {
  const raw = process.env?.[name]
  if (raw === undefined || raw === '') return fallback
  const parsed = Number(raw)
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback
}

// A cold host boots in tens of seconds and holds the request while it does.
// Aborting early kills the request that would have waited for the boot.
export function backendColdStartTimeoutMs(): number {
  return readEnvNumber('NUXT_BACKEND_WAKEUP_TIMEOUT_MS', 90_000)
}

export function backendProbeTimeoutMs(): number {
  return readEnvNumber('NUXT_BACKEND_WAKEUP_PROBE_TIMEOUT_MS', 10_000)
}

// Some edges (Render fronts its free services with a Cloudflare challenge)
// drop requests that present no browser identity, so the wake request never
// reaches the host and the host never boots. The default UA is therefore a
// plain browser string, which any ordinary server ignores; set the variable
// to an empty string to send nothing, or to your own value.
const DEFAULT_WAKEUP_USER_AGENT =
  'Mozilla/5.0 (X11; Linux x86_64; rv:158.0) Gecko/20100101 Firefox/158.0'

export function backendWakeupUserAgent(): string {
  return process.env?.NUXT_BACKEND_WAKEUP_USER_AGENT ?? DEFAULT_WAKEUP_USER_AGENT
}

function withWakeupIdentity(headers: Headers): Headers {
  const userAgent = backendWakeupUserAgent()
  const next = new Headers(headers)
  if (userAgent && !next.get('user-agent')) {
    next.set('user-agent', userAgent)
  }
  return next
}

export const BACKEND_FETCH_RETRY_STATUS_CODES = [
  408,
  409,
  425,
  500,
  502,
  503,
  504,
] as const

export type BackendFetchMethod =
  | 'GET'
  | 'HEAD'
  | 'PATCH'
  | 'POST'
  | 'PUT'
  | 'DELETE'
  | 'CONNECT'
  | 'OPTIONS'
  | 'TRACE'

export interface BackendFetchOptions {
  method: BackendFetchMethod
  headers: Headers
  body?: unknown
  credentials?: RequestCredentials
  ignoreResponseError?: boolean
  retry?: number
  retryStatusCodes?: readonly number[]
  timeout?: number
}

export interface BackendRawResponse<T = unknown> {
  status: number
  statusText: string
  headers: Headers
  _data: T
}

export interface BackendRequestContext {
  method: string
  path: string
  backendApiBase?: string
}

export interface BackendUnavailableResponse {
  error: {
    code: 'BACKEND_UNAVAILABLE'
    message: string
  }
}

function backendHealthUrls(backendApiBase: string): string[] {
  const normalizedBase = backendApiBase.replace(/\/+$/, '')

  // Probe both health bases so a cold Render instance is detected as soon
  // as the process is up (raw liveness, fast) while still allowing the
  // versioned readiness endpoint (DB + Redis) to gate full readiness.
  // The raw liveness endpoint is tried first because it returns 200 the
  // moment the process boots, before dependencies are connected.
  if (/\/api\/v\d+$/i.test(normalizedBase)) {
    const hostRoot = normalizedBase.replace(/\/api\/v\d+$/i, '')
    return [...new Set([`${hostRoot}/health`, `${normalizedBase}/health`])]
  }

  return [`${normalizedBase}/health`]
}

async function fetchBackendRaw<T>(
  targetUrl: string,
  options: BackendFetchOptions,
): Promise<BackendRawResponse<T>> {
  const response = await $fetch.raw<T>(targetUrl, {
    method: options.method,
    headers: options.headers,
    body: options.body as BodyInit | undefined,
    redirect: 'manual',
    credentials: options.credentials ?? 'include',
    ignoreResponseError: options.ignoreResponseError,
    ...(options.retry === undefined ? {} : { retry: options.retry }),
    ...(options.timeout === undefined ? {} : { timeout: options.timeout }),
    retryStatusCodes: options.retryStatusCodes
      ? [...options.retryStatusCodes]
      : [...BACKEND_FETCH_RETRY_STATUS_CODES],
  })

  return response as BackendRawResponse<T>
}

export async function fetchBackendWithRenderWakeupRetry<T = unknown>(
  targetUrl: string,
  backendApiBase: string,
  options: BackendFetchOptions,
  wakeupState: BackendWakeupState = createBackendWakeupState(),
): Promise<BackendRawResponse<T>> {
  let originalAttempt = 0

  while (originalAttempt < BACKEND_REQUEST_MAX_ATTEMPTS) {
    originalAttempt += 1

    try {
      return await fetchBackendRaw<T>(targetUrl, {
        ...options,
        headers: withWakeupIdentity(options.headers),
        timeout: options.timeout ?? backendColdStartTimeoutMs(),
      })
    } catch (error) {
      if (!isRenderWakeupError(error)) {
        throw error
      }

      if (originalAttempt >= BACKEND_REQUEST_MAX_ATTEMPTS) {
        throw error
      }

      if (!wakeupState.healthReady) {
        const healthUrls = backendHealthUrls(backendApiBase)
        await waitForBackendHealth(
          async () => {
            let lastResponse: BackendRawResponse<unknown> | undefined
            let lastError: unknown
            for (const healthUrl of healthUrls) {
              try {
                const response = await fetchBackendRaw<unknown>(
                  healthUrl,
                  {
                    method: 'GET',
                    headers: new Headers({
                      accept: 'text/html',
                      'user-agent': backendWakeupUserAgent(),
                    }),
                    credentials: 'omit',
                    ignoreResponseError: true,
                    retry: 0,
                    retryStatusCodes: [],
                    timeout: backendProbeTimeoutMs(),
                  },
                )
                if (response.status === 200) {
                  return response
                }
                lastResponse = response
              } catch (probeError) {
                lastError = probeError
              }
            }
            if (lastResponse) {
              return lastResponse
            }
            throw (lastError instanceof Error
              ? lastError
              : new Error('Backend health probe failed'))
          },
          { budget: wakeupState.healthBudget },
        )
        wakeupState.healthReady = true
      }
    }
  }

  throw new Error('Backend request attempts exhausted')
}

export function mapBackendWakeupExhaustion(
  event: H3Event,
  error: unknown,
  context: BackendRequestContext,
): BackendUnavailableResponse | null {
  if (
    !isRenderWakeupError(error)
    && !isBackendHealthProbeExhaustedError(error)
  ) {
    return null
  }

  console.warn(
    '[proxy] Render backend did not become ready:',
    JSON.stringify({
      event: 'backend_proxy.render_wakeup_timeout',
      method: context.method,
      path: context.path,
      backendApiBase: context.backendApiBase,
    }),
  )
  setResponseStatus(event, 503)
  setResponseHeader(event, 'retry-after', 30)

  return {
    error: {
      code: 'BACKEND_UNAVAILABLE',
      message: 'Backend API unavailable',
    },
  }
}
