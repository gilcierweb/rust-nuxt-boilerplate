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
        // Bound the initial attempt so a Render wake-up that holds the
        // connection open (browser-less requests) cannot hang the worker.
        timeout: options.timeout ?? 5000,
      })
    } catch (error) {
      if (!isRenderWakeupError(error)) {
        throw error
      }

      // Render is hibernating. Actively trigger the spin-up with a
      // browser-like request (Accept: text/html) so the service actually
      // wakes, then surface the wake-up to the caller (mapped to 503
      // BACKEND_UNAVAILABLE) and let the client retry with backoff.
      const healthUrls = backendHealthUrls(backendApiBase)
      for (const healthUrl of healthUrls) {
        try {
          await fetchBackendRaw<unknown>(healthUrl, {
            method: 'GET',
            headers: new Headers({ accept: 'text/html' }),
            credentials: 'omit',
            ignoreResponseError: true,
            retry: 0,
            retryStatusCodes: [],
            timeout: 2000,
          })
        } catch {
          // Best-effort trigger; ignore.
        }
      }
      throw error
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
