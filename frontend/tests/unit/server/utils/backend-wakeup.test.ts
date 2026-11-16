import { describe, expect, it } from 'vitest'

import * as backendWakeup from '../../../../server/utils/backend-wakeup'

const {
  fetchWithRenderWakeupRetry,
  isRenderWakeupResponse,
} = backendWakeup

function upstreamError(
  status: number,
  contentType: string,
  additionalHeaders: Record<string, string> = {},
): Error & { response: { status: number; headers: Headers } } {
  return Object.assign(new Error('Upstream request failed'), {
    response: {
      status,
      headers: new Headers({
        'content-type': contentType,
        ...additionalHeaders,
      }),
    },
  })
}

describe('isRenderWakeupResponse', () => {
  it('recognizes an HTML 429 without application rate-limit headers', () => {
    const error = upstreamError(429, 'text/html; charset=UTF-8')

    expect(isRenderWakeupResponse(error.response)).toBe(true)
  })

  it('recognizes a plain-text 429 without application rate-limit headers', () => {
    const error = upstreamError(429, 'text/plain; charset=utf-8')

    expect(isRenderWakeupResponse(error.response)).toBe(true)
  })

  it('does not recognize the backend JSON 429 as a Render wake-up response', () => {
    const error = upstreamError(429, 'application/json', {
      'x-ratelimit-limit': '10',
    })

    expect(isRenderWakeupResponse(error.response)).toBe(false)
  })

  it('does not recognize an HTML 429 that carries application rate-limit headers', () => {
    const error = upstreamError(429, 'text/html', {
      'x-ratelimit-limit': '10',
    })

    expect(isRenderWakeupResponse(error.response)).toBe(false)
  })

  it('does not recognize a plain-text 429 with another rate-limit header', () => {
    const error = upstreamError(429, 'text/plain', {
      'x-ratelimit-remaining': '9',
    })

    expect(isRenderWakeupResponse(error.response)).toBe(false)
  })

  it('requires an exact text media type', () => {
    const vendorError = upstreamError(429, 'application/vnd.text/plain')
    const suffixedHtmlError = upstreamError(429, 'text/htmlfoo')

    expect(isRenderWakeupResponse(vendorError.response)).toBe(false)
    expect(isRenderWakeupResponse(suffixedHtmlError.response)).toBe(false)
  })

  it('fails closed when rate-limit headers cannot be enumerated', () => {
    const response = {
      status: 429,
      headers: {
        get(name: string) {
          if (name === 'content-type') return 'text/plain'
          if (name === 'x-ratelimit-remaining') return '9'
          return null
        },
      },
    }

    expect(isRenderWakeupResponse(response)).toBe(false)
  })

  it('recognizes a Render hibernation 503 (text/html) as a wake-up response', () => {
    const error = upstreamError(503, 'text/html; charset=UTF-8')

    expect(isRenderWakeupResponse(error.response)).toBe(true)
  })
})

describe('backend health readiness', () => {
  it('retries every not-ready health outcome with the required delays', async () => {
    const delays: number[] = []
    let attempts = 0

    await expect(
      backendWakeup.waitForBackendHealth(
        async () => {
          attempts += 1
          if (attempts === 1) {
            throw new TypeError('fetch failed')
          }
          if (attempts === 2) {
            return { status: 503 }
          }
          if (attempts === 3) {
            return { status: 302 }
          }
          throw new TypeError('fetch failed')
        },
        {
          budget: backendWakeup.createBackendHealthBudget(),
          sleep: async (delayMs) => {
            delays.push(delayMs)
          },
        },
      ),
    ).rejects.toThrow('Backend health probe exhausted')

    expect(attempts).toBe(12)
    expect(delays).toEqual([3_000, 4_000, 5_000, 6_000, 8_000, 10_000, 12_000, 15_000, 20_000, 25_000, 30_000])
  })

  it('accepts only HTTP 200 as a ready health response', async () => {
    await expect(
      backendWakeup.waitForBackendHealth(
        async () => ({ status: 204 }),
        {
          budget: backendWakeup.createBackendHealthBudget(),
          sleep: async () => {},
        },
      ),
    ).rejects.toThrow('Backend health probe exhausted')
  })
})

describe('fetchWithRenderWakeupRetry', () => {
  it('returns the backend response after a Render wake-up response', async () => {
    const wakeupError = upstreamError(429, 'text/html; charset=UTF-8')
    const expectedResponse = { status: 401, body: { error: 'invalid credentials' } }
    let attempts = 0

    const response = await fetchWithRenderWakeupRetry(
      async () => {
        attempts += 1
        if (attempts === 1) {
          throw wakeupError
        }
        return expectedResponse
      },
      {
        maxAttempts: 3,
        sleep: async () => {},
      },
    )

    expect(response).toBe(expectedResponse)
    expect(attempts).toBe(2)
  })

  it('does not retry the backend rate-limit response', async () => {
    const rateLimitError = upstreamError(429, 'application/json', {
      'x-ratelimit-limit': '10',
    })
    let attempts = 0

    await expect(
      fetchWithRenderWakeupRetry(
        async () => {
          attempts += 1
          throw rateLimitError
        },
        {
          maxAttempts: 3,
          sleep: async () => {},
        },
      ),
    ).rejects.toBe(rateLimitError)

    expect(attempts).toBe(1)
  })

  it('stops after the configured attempt budget and propagates the last error', async () => {
    const wakeupError = upstreamError(429, 'text/html; charset=UTF-8')
    let attempts = 0

    await expect(
      fetchWithRenderWakeupRetry(
        async () => {
          attempts += 1
          throw wakeupError
        },
        {
          maxAttempts: 3,
          sleep: async () => {},
        },
      ),
    ).rejects.toBe(wakeupError)

    expect(attempts).toBe(3)
  })

  it('requires an explicit attempt budget', async () => {
    const wakeupError = upstreamError(429, 'text/plain; charset=utf-8')
    let attempts = 0

    await expect(
      fetchWithRenderWakeupRetry(
        async () => {
          attempts += 1
          throw wakeupError
        },
        {
          maxAttempts: 3,
          sleep: async () => {},
        },
      ),
    ).rejects.toBe(wakeupError)

    expect(attempts).toBe(3)
  })

  it('uses capped exponential delays between wake-up attempts', async () => {
    const wakeupError = upstreamError(429, 'text/html; charset=UTF-8')
    const delays: number[] = []
    let attempts = 0

    await fetchWithRenderWakeupRetry(
      async () => {
        attempts += 1
        if (attempts < 4) {
          throw wakeupError
        }
        return 'ready'
      },
      {
        maxAttempts: 4,
        initialDelayMs: 100,
        maxDelayMs: 250,
        sleep: async (delayMs) => {
          delays.push(delayMs)
        },
      },
    )

    expect(delays).toEqual([100, 200, 250])
  })
})
