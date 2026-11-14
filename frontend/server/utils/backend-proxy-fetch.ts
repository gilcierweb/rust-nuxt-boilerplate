import {
  setResponseHeader,
  setResponseStatus,
  type H3Event,
} from 'h3'

import {
  fetchWithRenderWakeupRetry,
  isRenderWakeupError,
} from '~~/server/utils/backend-wakeup'

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

  return `${normalizedBase}/api/v1/health`
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
    credentials: 'include',
    retryStatusCodes: [...BACKEND_FETCH_RETRY_STATUS_CODES],
  })

  return response as BackendRawResponse<T>
}

export async function fetchBackendWithRenderWakeupRetry<T = unknown>(
  targetUrl: string,
  backendApiBase: string,
  options: BackendFetchOptions,
): Promise<BackendRawResponse<T>> {
  const waitUntilReady = async () => {
    await fetchWithRenderWakeupRetry(
      () => fetchBackendRaw(
        backendHealthUrl(backendApiBase),
        {
          method: 'GET',
          headers: new Headers({ accept: 'application/json' }),
        },
      ),
      {
        maxAttempts: 4,
        initialDelayMs: 2_000,
        maxDelayMs: 8_000,
      },
    )
  }

  return fetchWithRenderWakeupRetry(
    () => fetchBackendRaw<T>(targetUrl, options),
    {
      maxAttempts: 3,
      waitUntilReady,
    },
  )
}

export function mapBackendWakeupExhaustion(
  event: H3Event,
  error: unknown,
  context: BackendRequestContext,
): BackendUnavailableResponse | null {
  if (!isRenderWakeupError(error)) {
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
