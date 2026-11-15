import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { ofetch } from 'ofetch'

import { fetchBackendWithRenderWakeupRetry } from '../../../../server/utils/backend-proxy-fetch'
import { createBackendWakeupState } from '../../../../server/utils/backend-wakeup'

function jsonResponse(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
}

function platformRateLimitResponse(): Response {
  return new Response('Too Many Requests', {
    status: 429,
    headers: { 'content-type': 'text/plain; charset=utf-8' },
  })
}

function installOfetch(fakeFetch: typeof globalThis.fetch): void {
  const fetchInstance = ofetch.create({}, { fetch: fakeFetch })
  vi.stubGlobal('$fetch', fetchInstance)
}

describe('fetchBackendWithRenderWakeupRetry with real ofetch', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
    vi.unstubAllGlobals()
  })

  it('preserves the automatic ofetch retry for an eligible non-429 failure', async () => {
    const requestedUrls: string[] = []
    const fakeFetch = vi.fn(async (request: RequestInfo | URL) => {
      requestedUrls.push(String(request))
      if (requestedUrls.length === 1) {
        return jsonResponse(503, { status: 'unavailable' })
      }

      return jsonResponse(200, { status: 'ok' })
    })
    installOfetch(fakeFetch as typeof globalThis.fetch)

    const responsePromise = fetchBackendWithRenderWakeupRetry(
      'https://backend.example/api/v1/items',
      'https://backend.example/api/v1',
      {
        method: 'GET',
        headers: new Headers({ accept: 'application/json' }),
      },
    )
    await vi.runAllTimersAsync()
    const response = await responsePromise

    expect(requestedUrls).toEqual([
      'https://backend.example/api/v1/items',
      'https://backend.example/api/v1/items',
    ])
    expect(response.status).toBe(200)
  })

  it('does not retry or probe for a JSON 429 without rate-limit headers', async () => {
    const requestedUrls: string[] = []
    const fakeFetch = vi.fn(async (request: RequestInfo | URL) => {
      requestedUrls.push(String(request))
      return jsonResponse(429, {
        error: {
          code: 'RATE_LIMITED',
          message: 'Rate limit exceeded',
        },
      })
    })
    installOfetch(fakeFetch as typeof globalThis.fetch)

    await expect(fetchBackendWithRenderWakeupRetry(
      'https://backend.example/api/v1/items',
      'https://backend.example/api/v1',
      {
        method: 'GET',
        headers: new Headers({ accept: 'application/json' }),
      },
    )).rejects.toMatchObject({
      response: {
        status: 429,
      },
    })

    expect(requestedUrls).toEqual([
      'https://backend.example/api/v1/items',
    ])
  })

  it('uses one readiness cycle and reuses it for later fallback candidates', async () => {
    const requestedUrls: string[] = []
    const attemptsByUrl = new Map<string, number>()
    const fakeFetch = vi.fn(async (request: RequestInfo | URL) => {
      const url = String(request)
      requestedUrls.push(url)
      const attempt = (attemptsByUrl.get(url) ?? 0) + 1
      attemptsByUrl.set(url, attempt)

      if (url === 'https://backend.example/health' || url === 'https://backend.example/api/v1/health') {
        return jsonResponse(200, { status: 'ok' })
      }

      if (attempt === 1) {
        return platformRateLimitResponse()
      }

      return jsonResponse(200, { status: 'ok' })
    })
    installOfetch(fakeFetch as typeof globalThis.fetch)
    const wakeupState = createBackendWakeupState()
    const requestOptions = {
      method: 'GET' as const,
      headers: new Headers({ accept: 'application/json' }),
    }

    const firstCandidatePromise = fetchBackendWithRenderWakeupRetry(
      'https://backend.example/api/v1/items',
      'https://backend.example/api/v1',
      requestOptions,
      wakeupState,
    )
    await vi.runAllTimersAsync()
    await firstCandidatePromise

    const fallbackCandidatePromise = fetchBackendWithRenderWakeupRetry(
      'https://backup.example/api/v1/items',
      'https://backup.example/api/v1',
      requestOptions,
      wakeupState,
    )
    await vi.runAllTimersAsync()
    await fallbackCandidatePromise

    expect(requestedUrls).toEqual([
      'https://backend.example/api/v1/items',
      'https://backend.example/health',
      'https://backend.example/api/v1/items',
      'https://backup.example/api/v1/items',
      'https://backup.example/api/v1/items',
    ])
  })

  it('does not let ofetch internally retry a failed health probe', async () => {
    const requestedUrls: string[] = []
    const originalAttempts: string[] = []
    const healthAttempts: string[] = []
    const fakeFetch = vi.fn(async (request: RequestInfo | URL) => {
      const url = String(request)
      requestedUrls.push(url)

      if (url === 'https://backend.example/api/v1/items') {
        originalAttempts.push(url)
        if (originalAttempts.length === 1) {
          return platformRateLimitResponse()
        }

        return jsonResponse(200, { status: 'ok' })
      }

      if (url === 'https://backend.example/health' || url === 'https://backend.example/api/v1/health') {
        const rawHealthCalls = healthAttempts.filter((value) => value === 'https://backend.example/health').length
        healthAttempts.push(url)
        if (url === 'https://backend.example/health' && rawHealthCalls === 0) {
          throw new TypeError('fetch failed')
        }

        if (url === 'https://backend.example/health') {
          return jsonResponse(200, { status: 'ok' })
        }

        return jsonResponse(503, { status: 'unavailable' })
      }

      throw new Error(`Unexpected URL: ${url}`)
    })
    installOfetch(fakeFetch as typeof globalThis.fetch)

    const responsePromise = fetchBackendWithRenderWakeupRetry(
      'https://backend.example/api/v1/items',
      'https://backend.example/api/v1',
      {
        method: 'GET',
        headers: new Headers({ accept: 'application/json' }),
      },
    )
    await vi.runAllTimersAsync()
    const response = await responsePromise

    expect(requestedUrls).toEqual([
      'https://backend.example/api/v1/items',
      'https://backend.example/health',
      'https://backend.example/api/v1/health',
      'https://backend.example/health',
      'https://backend.example/api/v1/items',
    ])
    expect(response.status).toBe(200)
  })
})
