type HeaderReader = {
  get(name: string): string | null
  forEach?: (callback: (value: string, name: string) => void) => void
}

type ResponseWithHeaders = {
  status?: unknown
  headers?: unknown
}

export interface BackendWakeupRetryOptions {
  maxAttempts?: number
  initialDelayMs?: number
  maxDelayMs?: number
  sleep?: (delayMs: number) => Promise<void>
  waitUntilReady?: () => Promise<void>
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function readHeader(headers: unknown, name: string): string | null {
  if (!isRecord(headers)) {
    return null
  }

  const reader = headers as HeaderReader
  if (typeof reader.get !== 'function') {
    return null
  }

  return reader.get(name)
}

function hasRateLimitHeader(headers: unknown): boolean {
  if (!isRecord(headers)) {
    return false
  }

  const reader = headers as HeaderReader
  if (typeof reader.forEach === 'function') {
    let hasRateLimitHeader = false
    reader.forEach((_value, name) => {
      if (name.toLowerCase().startsWith('x-ratelimit-')) {
        hasRateLimitHeader = true
      }
    })
    return hasRateLimitHeader
  }

  return true
}

function responseFromError(error: unknown): unknown {
  if (!isRecord(error)) {
    return undefined
  }

  return error.response
}

export function isRenderWakeupError(error: unknown): boolean {
  return isRenderWakeupResponse(responseFromError(error))
}

export function isRenderWakeupResponse(response: unknown): boolean {
  if (!isRecord(response)) {
    return false
  }

  const typedResponse = response as ResponseWithHeaders
  if (typedResponse.status !== 429) {
    return false
  }

  const contentType = readHeader(typedResponse.headers, 'content-type')
  const mediaType = contentType?.split(';', 1)[0]?.trim().toLowerCase()
  if (mediaType !== 'text/html' && mediaType !== 'text/plain') {
    return false
  }

  return !hasRateLimitHeader(typedResponse.headers)
}

function defaultSleep(delayMs: number): Promise<void> {
  return new Promise((resolve) => {
    setTimeout(resolve, delayMs)
  })
}

export async function fetchWithRenderWakeupRetry<T>(
  operation: () => Promise<T>,
  options: BackendWakeupRetryOptions = {},
): Promise<T> {
  const maxAttempts = Math.max(1, Math.floor(options.maxAttempts ?? 7))
  const initialDelayMs = Math.max(0, Math.floor(options.initialDelayMs ?? 2_000))
  const maxDelayMs = Math.max(0, Math.floor(options.maxDelayMs ?? 30_000))
  const sleep = options.sleep ?? defaultSleep

  const executeWithRetry = async (attempt: number): Promise<T> => {
    try {
      return await operation()
    } catch (error) {
      if (
        attempt >= maxAttempts
        || !isRenderWakeupError(error)
      ) {
        throw error
      }

      const delayMs = Math.min(
        initialDelayMs * 2 ** (attempt - 1),
        maxDelayMs,
      )
      if (options.waitUntilReady) {
        await options.waitUntilReady()
      } else {
        await sleep(delayMs)
      }
      return executeWithRetry(attempt + 1)
    }
  }

  return executeWithRetry(1)
}
