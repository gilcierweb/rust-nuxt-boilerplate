import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  appendResponseHeader: vi.fn(),
  createError: vi.fn((options: Record<string, unknown>) =>
    Object.assign(new Error(String(options.statusMessage ?? 'Error')), options),
  ),
  getRequestHeaders: vi.fn(() => ({})),
  getRequestURL: vi.fn(() => new URL('https://worker.test/api/v1/auth/login')),
  readRawBody: vi.fn(async () => new TextEncoder().encode('{}')),
  setResponseHeader: vi.fn(),
  setResponseStatus: vi.fn(),
  rawFetch: vi.fn(),
  resolveAccessTokenForProxy: vi.fn(async () => null),
}))

vi.mock('h3', () => ({
  appendResponseHeader: mocks.appendResponseHeader,
  createError: mocks.createError,
  getRequestHeaders: mocks.getRequestHeaders,
  getRequestURL: mocks.getRequestURL,
  readRawBody: mocks.readRawBody,
  setResponseHeader: mocks.setResponseHeader,
  setResponseStatus: mocks.setResponseStatus,
}))

vi.mock('~~/server/utils/auth', () => ({
  resolveAccessTokenForProxy: mocks.resolveAccessTokenForProxy,
}))

type ProxyHandler = (event: unknown) => Promise<unknown>

let proxyHandler: ProxyHandler
let sessionHandler: ProxyHandler
let refreshHandler: ProxyHandler

function wakeupError(): Error & {
  response: {
    status: number
    statusText: string
    headers: Headers
    _data: string
  }
} {
  return Object.assign(new Error('Too Many Requests'), {
    response: {
      status: 429,
      statusText: 'Too Many Requests',
      headers: new Headers({
        'content-type': 'text/plain; charset=utf-8',
      }),
      _data: '<html>Too Many Requests</html>',
    },
  })
}

function backendRateLimitError(): Error & {
  response: {
    status: number
    statusText: string
    headers: Headers
    _data: {
      error: {
        code: string
        message: string
      }
    }
  }
} {
  return Object.assign(new Error('Too Many Requests'), {
    response: {
      status: 429,
      statusText: 'Too Many Requests',
      headers: new Headers({
        'content-type': 'application/json',
        'x-ratelimit-limit': '10',
      }),
      _data: {
        error: {
          code: 'RATE_LIMITED',
          message: 'Rate limit exceeded',
        },
      },
    },
  })
}

beforeAll(async () => {
  vi.stubGlobal('defineEventHandler', (handler: ProxyHandler) => handler)
  vi.stubGlobal('useRuntimeConfig', () => ({
    backendApiBase: 'https://backend.example/api/v1',
    backendApiKey: 'test-api-key',
  }))
  vi.stubGlobal('$fetch', { raw: mocks.rawFetch })

  const [route, sessionRoute, refreshRoute] = await Promise.all([
    import('../../../../server/api/v1/[...path].ts'),
    import('../../../../server/api/v1/auth/session.get.ts'),
    import('../../../../server/api/v1/auth/refresh.post.ts'),
  ])
  proxyHandler = route.default
  sessionHandler = sessionRoute.default
  refreshHandler = refreshRoute.default
})

beforeEach(() => {
  vi.useFakeTimers()
  vi.clearAllMocks()
  vi.spyOn(console, 'warn').mockImplementation(() => {})
})

afterEach(() => {
  vi.useRealTimers()
})

describe('backend proxy route', () => {
  it('returns the backend response after the Render service finishes waking up', async () => {
    const expectedResponse = {
      error: {
        code: 'UNAUTHORIZED',
        message: 'Invalid credentials',
      },
    }
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce({
        status: 200,
        statusText: 'OK',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: { status: 'ok' },
      })
      .mockResolvedValueOnce({
        status: 401,
        statusText: 'Unauthorized',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: expectedResponse,
      })

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const responsePromise = proxyHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    expect(mocks.rawFetch).toHaveBeenCalledTimes(3)
    expect(mocks.rawFetch.mock.calls[1][0]).toBe(
      'https://backend.example/api/v1/health',
    )
    expect(mocks.rawFetch.mock.calls[1][1].headers.has('x-api-key')).toBe(false)
    expect(mocks.rawFetch.mock.calls[1][1].headers.has('cookie')).toBe(false)
    expect(response).toEqual(expectedResponse)
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(
      event,
      401,
      'Unauthorized',
    )
  })

  it('returns 503 when the Render service does not finish waking up in time', async () => {
    mocks.rawFetch.mockRejectedValue(wakeupError())

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const responsePromise = proxyHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise as {
      error: { code: string }
    }

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(mocks.rawFetch).toHaveBeenCalledTimes(5)
    expect(healthCalls).toHaveLength(4)
    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })

  it('passes through the backend rate-limit response without retrying it', async () => {
    const rateLimitError = backendRateLimitError()
    mocks.rawFetch.mockRejectedValue(rateLimitError)

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const response = await proxyHandler(event)

    expect(mocks.rawFetch).toHaveBeenCalledTimes(1)
    expect(mocks.rawFetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        retryStatusCodes: [408, 409, 425, 500, 502, 503, 504],
      }),
    )
    expect(mocks.rawFetch.mock.calls[0][1]).not.toHaveProperty('retry')
    expect(response).toEqual(rateLimitError.response._data)
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(
      event,
      429,
      'Too Many Requests',
    )
  })

  it('waits for backend health before retrying the session request', async () => {
    const expectedResponse = { user: null }
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce({
        status: 200,
        statusText: 'OK',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: { status: 'ok' },
      })
      .mockResolvedValueOnce({
        status: 200,
        statusText: 'OK',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: expectedResponse,
      })

    const event = {
      method: 'GET',
      context: { params: {} },
    }
    const responsePromise = sessionHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    expect(mocks.rawFetch).toHaveBeenCalledTimes(3)
    expect(mocks.rawFetch.mock.calls[1][0]).toBe(
      'https://backend.example/api/v1/health',
    )
    expect(response).toEqual(expectedResponse)
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 200, 'OK')
  })

  it('waits for backend health before retrying the refresh request', async () => {
    const expectedResponse = { access_token: 'test-token' }
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce({
        status: 200,
        statusText: 'OK',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: { status: 'ok' },
      })
      .mockResolvedValueOnce({
        status: 200,
        statusText: 'OK',
        headers: new Headers({ 'content-type': 'application/json' }),
        _data: expectedResponse,
      })

    const event = {
      method: 'POST',
      context: { params: {} },
    }
    const responsePromise = refreshHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    expect(mocks.rawFetch).toHaveBeenCalledTimes(3)
    expect(mocks.rawFetch.mock.calls[1][0]).toBe(
      'https://backend.example/api/v1/health',
    )
    expect(response).toEqual(expectedResponse)
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 200, 'OK')
  })

  it('returns 503 when a dedicated auth route cannot wake the backend', async () => {
    mocks.rawFetch.mockRejectedValue(wakeupError())

    const event = {
      method: 'GET',
      context: { params: {} },
    }
    const responsePromise = sessionHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise as {
      error: { code: string }
    }

    expect(mocks.rawFetch).toHaveBeenCalledTimes(5)
    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })
})
