type HeaderReader = {
  get(name: string): string | null
  forEach?: (callback: (value: string, name: string) => void) => void
}

type ResponseWithHeaders = {
  status?: unknown
  headers?: unknown
}

export interface BackendWakeupRetryOptions {
  maxAttempts: number
  initialDelayMs?: number
  maxDelayMs?: number
  sleep?: (delayMs: number) => Promise<void>
}

export const BACKEND_HEALTH_MAX_ATTEMPTS = 4
export const BACKEND_HEALTH_DELAYS_MS = [2_000, 4_000, 8_000] as const

export interface BackendHealthBudget {
  attemptsUsed: number
}

export interface BackendWakeupState {
  healthBudget: BackendHealthBudget
  healthReady: boolean
}

export interface BackendHealthWaitOptions {
  budget: BackendHealthBudget
  sleep?: (delayMs: number) => Promise<void>
}

export class BackendHealthProbeExhaustedError extends Error {
  readonly code = 'BACKEND_HEALTH_PROBE_EXHAUSTED'

  constructor(readonly attempts: number) {
    super('Backend health probe exhausted')
    this.name = 'BackendHealthProbeExhaustedError'
  }
}

export function createBackendHealthBudget(): BackendHealthBudget {
  return { attemptsUsed: 0 }
}

export function createBackendWakeupState(): BackendWakeupState {
  return {
    healthBudget: createBackendHealthBudget(),
    healthReady: false,
  }
}

export function isBackendHealthProbeExhaustedError(
  error: unknown,
): error is BackendHealthProbeExhaustedError {
  return error instanceof BackendHealthProbeExhaustedError
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

export async function waitForBackendHealth<T extends { status: number }>(
  operation: () => Promise<T>,
  options: BackendHealthWaitOptions,
): Promise<T> {
  const sleep = options.sleep ?? defaultSleep
  const maxAttempts = Math.max(1, Math.floor(BACKEND_HEALTH_MAX_ATTEMPTS))

  while (options.budget.attemptsUsed < maxAttempts) {
    options.budget.attemptsUsed += 1

    try {
      const response = await operation()
      if (response.status === 200) {
        return response
      }
    } catch {
      // Network and HTTP failures consume the shared readiness budget.
    }

    if (options.budget.attemptsUsed < maxAttempts) {
      const delayIndex = options.budget.attemptsUsed - 1
      await sleep(BACKEND_HEALTH_DELAYS_MS[delayIndex] ?? 0)
    }
  }

  throw new BackendHealthProbeExhaustedError(options.budget.attemptsUsed)
}

export async function fetchWithRenderWakeupRetry<T>(
  operation: () => Promise<T>,
  options: BackendWakeupRetryOptions,
): Promise<T> {
  const maxAttempts = Math.max(1, Math.floor(options.maxAttempts))
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
      await sleep(delayMs)
      return executeWithRetry(attempt + 1)
    }
  }

  return executeWithRetry(1)
}
