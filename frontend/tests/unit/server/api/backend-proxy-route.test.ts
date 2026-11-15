import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  appendResponseHeader: vi.fn(),
  createError: vi.fn((options: Record<string, unknown>) =>
    Object.assign(new Error(String(options.statusMessage ?? 'Error')), options),
  ),
  getRequestHeaders: vi.fn(() => ({})),
  getRequestURL: vi.fn(() => new URL('https://worker.test/api/v1/auth/login')),
  readRawBody: vi.fn(async () => new TextEncoder().encode('{}')),
  runtimeConfig: {
    backendApiBase: 'https://backend.example/api/v1',
    backendApiKey: 'test-api-key',
  },
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

function backendRateLimitError(
  additionalHeaders: Record<string, string> = {
    'x-ratelimit-limit': '10',
  },
): Error & {
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
        ...additionalHeaders,
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

function backendResponseError(
  status: number,
  statusText: string,
  contentType = 'application/json',
): Error & {
  response: {
    status: number
    statusText: string
    headers: Headers
    _data: { status: string }
  }
} {
  return Object.assign(new Error(statusText), {
    response: {
      status,
      statusText,
      headers: new Headers({ 'content-type': contentType }),
      _data: { status: 'unavailable' },
    },
  })
}

function successfulResponse<T>(data: T, status = 200): {
  status: number
  statusText: string
  headers: Headers
  _data: T
} {
  return {
    status,
    statusText: status === 200 ? 'OK' : 'Found',
    headers: new Headers({ 'content-type': 'application/json' }),
    _data: data,
  }
}

beforeAll(async () => {
  vi.stubGlobal('defineEventHandler', (handler: ProxyHandler) => handler)
  vi.stubGlobal('useRuntimeConfig', () => mocks.runtimeConfig)
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
  mocks.rawFetch.mockReset()
  mocks.runtimeConfig.backendApiBase = 'https://backend.example/api/v1'
  mocks.runtimeConfig.backendApiKey = 'test-api-key'
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
      'https://backend.example/health',
    )
    expect(mocks.rawFetch.mock.calls[1][1].headers.has('x-api-key')).toBe(false)
    expect(mocks.rawFetch.mock.calls[1][1].headers.has('authorization')).toBe(false)
    expect(mocks.rawFetch.mock.calls[1][1].headers.has('cookie')).toBe(false)
    expect(mocks.rawFetch.mock.calls[1][1].credentials).toBe('omit')
    expect(mocks.rawFetch.mock.calls[1][1].retry).toBe(0)
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
    expect(mocks.rawFetch).toHaveBeenCalledTimes(9)
    expect(healthCalls).toHaveLength(8)
    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })

  it('polls readiness with backoff until the health endpoint returns 200', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(backendResponseError(503, 'Service Unavailable'))
      .mockRejectedValueOnce(new TypeError('fetch failed'))
      .mockResolvedValueOnce(successfulResponse({ status: 'ok' }, 200))
      .mockResolvedValueOnce(successfulResponse({ user: null }))

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const responsePromise = proxyHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(healthCalls).toHaveLength(3)
    expect(response).toEqual({ user: null })
  })

  it('does not treat a redirect response as backend readiness', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce(successfulResponse({ location: '/login' }, 302))
      .mockResolvedValueOnce(successfulResponse({ status: 'ok' }, 200))
      .mockResolvedValueOnce(successfulResponse({ user: null }))

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const responsePromise = proxyHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(healthCalls).toHaveLength(2)
    expect(response).toEqual({ user: null })
  })

  it('shares one health budget across all original-request attempts', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(backendResponseError(503, 'Service Unavailable'))
      .mockResolvedValueOnce(successfulResponse({ status: 'ok' }, 200))
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce(successfulResponse({ user: null }))

    const event = {
      context: {
        params: { path: 'auth/login' },
      },
    }
    const responsePromise = proxyHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(mocks.rawFetch).toHaveBeenCalledTimes(5)
    expect(healthCalls).toHaveLength(2)
    expect(response).toEqual({ user: null })
  })

  it('maps readiness exhaustion to 503 even when the final health failure is not a 429', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(backendResponseError(503, 'Service Unavailable'))
      .mockRejectedValueOnce(new TypeError('fetch failed'))
      .mockResolvedValueOnce(successfulResponse({ status: 'degraded' }, 503))
      .mockRejectedValueOnce(new TypeError('fetch failed'))

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

    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })

  it('never exceeds three original-request attempts', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce(successfulResponse({ status: 'ok' }, 200))
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(wakeupError())

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

    const originalCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      !String(url).endsWith('/health'),
    )
    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(originalCalls).toHaveLength(3)
    expect(healthCalls).toHaveLength(1)
    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
  })

  it.each([
    {
      name: 'with rate-limit headers',
      error: backendRateLimitError(),
    },
    {
      name: 'without rate-limit headers',
      error: backendRateLimitError({}),
    },
  ])('passes through a JSON rate-limit response $name without retrying it', async ({ error: rateLimitError }) => {
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
      'https://backend.example/health',
    )
    expect(mocks.rawFetch.mock.calls[1][1].credentials).toBe('omit')
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
      'https://backend.example/health',
    )
    expect(mocks.rawFetch.mock.calls[1][1].credentials).toBe('omit')
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

    expect(mocks.rawFetch).toHaveBeenCalledTimes(9)
    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })

  it('maps session readiness exhaustion to 503 after a non-429 failure', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(backendResponseError(503, 'Service Unavailable'))
      .mockRejectedValueOnce(new TypeError('fetch failed'))
      .mockResolvedValueOnce(successfulResponse({ status: 'degraded' }, 503))
      .mockRejectedValueOnce(new TypeError('fetch failed'))

    const event = {
      method: 'GET',
      context: { params: {} },
    }
    const responsePromise = sessionHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise as {
      error: { code: string }
    }

    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
  })

  it('maps refresh readiness exhaustion to 503 after a non-429 failure', async () => {
    mocks.rawFetch
      .mockRejectedValueOnce(wakeupError())
      .mockRejectedValueOnce(backendResponseError(503, 'Service Unavailable'))
      .mockRejectedValueOnce(new TypeError('fetch failed'))
      .mockResolvedValueOnce(successfulResponse({ status: 'degraded' }, 503))
      .mockRejectedValueOnce(new TypeError('fetch failed'))

    const event = {
      method: 'POST',
      context: { params: {} },
    }
    const responsePromise = refreshHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise as {
      error: { code: string }
    }

    expect(response.error.code).toBe('BACKEND_UNAVAILABLE')
    expect(mocks.setResponseStatus).toHaveBeenLastCalledWith(event, 503)
    expect(mocks.setResponseHeader).toHaveBeenLastCalledWith(
      event,
      'retry-after',
      30,
    )
  })

  it('reuses readiness across fallback URLs in the same request', async () => {
    const attempts = new Map<string, number>()
    mocks.rawFetch.mockImplementation(async (url: string) => {
      if (url === 'https://backend.example/health' || url === 'https://backend.example/api/v1/health') {
        return successfulResponse({ status: 'ok' }, 200)
      }

      const attempt = (attempts.get(url) ?? 0) + 1
      attempts.set(url, attempt)

      if (url === 'https://backend.example/api/v1/auth/refresh') {
        if (attempt === 1) {
          throw wakeupError()
        }
        throw backendResponseError(404, 'Not Found')
      }

      if (url === 'https://backend.example/api/v1/auth/refresh/') {
        if (attempt === 1) {
          throw wakeupError()
        }
        return successfulResponse({ access_token: 'test-token' })
      }

      throw new Error(`Unexpected backend URL: ${url}`)
    })

    const event = {
      method: 'POST',
      context: { params: {} },
    }
    const responsePromise = refreshHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(healthCalls).toHaveLength(1)
    expect(response).toEqual({ access_token: 'test-token' })
  })

  it('uses the raw liveness endpoint for a raw backend base', async () => {
    mocks.runtimeConfig.backendApiBase = 'https://backend.example'
    mocks.rawFetch
      .mockRejectedValueOnce(backendResponseError(404, 'Not Found'))
      .mockRejectedValueOnce(backendResponseError(404, 'Not Found'))
      .mockRejectedValueOnce(wakeupError())
      .mockResolvedValueOnce(successfulResponse({ status: 'ok' }, 200))
      .mockResolvedValueOnce(successfulResponse({ access_token: 'test-token' }))

    const event = {
      method: 'POST',
      context: { params: {} },
    }
    const responsePromise = refreshHandler(event)
    await vi.runAllTimersAsync()
    const response = await responsePromise

    const healthCalls = mocks.rawFetch.mock.calls.filter(([url]) =>
      String(url).endsWith('/health'),
    )
    expect(healthCalls.every(([url]) =>
      url === 'https://backend.example/health',
    )).toBe(true)
    expect(response).toEqual({ access_token: 'test-token' })
  })
})
