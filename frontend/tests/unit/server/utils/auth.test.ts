import { describe, it, expect, vi } from 'vitest'

// h3 is a Nitro/server runtime package not available in the vitest jsdom env.
// Mock it before importing the module under test.
vi.mock('h3', () => ({
  getRequestHeaders: vi.fn(() => ({})),
}))

import {
  parseBearerToken,
  classifySessionFetchError,
} from '../../../../server/utils/auth'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function fetchError(status?: number): Error & { response?: { status?: number } } {
  const err = new Error('fetch error') as Error & { response?: { status?: number } }
  if (status !== undefined) {
    err.response = { status }
  }
  return err
}

// ---------------------------------------------------------------------------
// parseBearerToken
// ---------------------------------------------------------------------------

describe('parseBearerToken', () => {
  it('extracts token from valid Bearer header', () => {
    expect(parseBearerToken('Bearer my-token-123')).toBe('my-token-123')
  })

  it('is case-insensitive on the "bearer" prefix', () => {
    expect(parseBearerToken('BEARER my-token')).toBe('my-token')
    expect(parseBearerToken('bearer my-token')).toBe('my-token')
    expect(parseBearerToken('BeArEr my-token')).toBe('my-token')
  })

  it('trims whitespace from the extracted token', () => {
    expect(parseBearerToken('Bearer   spaced-token  ')).toBe('spaced-token')
  })

  it('returns null for an empty string', () => {
    expect(parseBearerToken('')).toBeNull()
  })

  it('returns null for undefined', () => {
    expect(parseBearerToken(undefined)).toBeNull()
  })

  it('returns null for non-Bearer schemes', () => {
    expect(parseBearerToken('Basic dXNlcjpwYXNz')).toBeNull()
    expect(parseBearerToken('Digest realm="example"')).toBeNull()
  })

  it('returns null for "Bearer " with an empty token', () => {
    expect(parseBearerToken('Bearer ')).toBeNull()
    expect(parseBearerToken('Bearer   ')).toBeNull()
  })
})

// ---------------------------------------------------------------------------
// classifySessionFetchError
// ---------------------------------------------------------------------------

describe('classifySessionFetchError', () => {
  it('returns null on 401 (expired / invalid refresh token — expected)', () => {
    expect(classifySessionFetchError(fetchError(401))).toBeNull()
  })

  it('returns null on 404 (session endpoint not found on replica — expected)', () => {
    expect(classifySessionFetchError(fetchError(404))).toBeNull()
  })

  it('re-throws on 500 — must not silently degrade auth', () => {
    expect(() => classifySessionFetchError(fetchError(500))).toThrow()
  })

  it('re-throws on 502 (backend gateway down)', () => {
    expect(() => classifySessionFetchError(fetchError(502))).toThrow()
  })

  it('re-throws on 503 (backend temporarily unavailable)', () => {
    expect(() => classifySessionFetchError(fetchError(503))).toThrow()
  })

  it('re-throws on 504 (gateway timeout)', () => {
    expect(() => classifySessionFetchError(fetchError(504))).toThrow()
  })

  it('re-throws on network error (no response / no status)', () => {
    expect(() => classifySessionFetchError(fetchError(undefined))).toThrow()
  })

  it('re-throws a plain Error with no response property', () => {
    expect(() => classifySessionFetchError(new Error('network failure'))).toThrow('network failure')
  })

  it('preserves the original error when re-throwing', () => {
    const original = fetchError(503)
    expect(() => classifySessionFetchError(original)).toThrow(original)
  })

  it('re-throws on 400 (bad request — not an expected auth failure)', () => {
    expect(() => classifySessionFetchError(fetchError(400))).toThrow()
  })
})
