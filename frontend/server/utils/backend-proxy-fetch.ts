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
}

export interface BackendUnavailableResponse {
  error: {
    code: 'BACKEND_UNAVAILABLE'
    message: string
  }
}

function backendHealthUrl(backendApiBase: string): string {
  const normalizedBase = backendApiBase.replace(/\/+$/, '')
  if (/\/api\/v\d+$/i.test(normalizedBase)) {
    return `${normalizedBase}/health`
  }

  return `${normalizedBase}/health`
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
      return await fetchBackendRaw<T>(targetUrl, options)
    } catch (error) {
      if (!isRenderWakeupError(error)) {
        throw error
      }

      if (originalAttempt >= BACKEND_REQUEST_MAX_ATTEMPTS) {
        throw error
      }

      if (!wakeupState.healthReady) {
        await waitForBackendHealth(
          () => fetchBackendRaw(
            backendHealthUrl(backendApiBase),
            {
              method: 'GET',
              headers: new Headers({ accept: 'application/json' }),
              credentials: 'omit',
              ignoreResponseError: true,
              retry: 0,
              retryStatusCodes: [],
            },
          ),
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
