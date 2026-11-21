import { describe, expect, it, vi } from 'vitest'
import { mapAuthError } from '~/utils/auth-errors'

// The function that decides what an auth failure says to the user: it reads
// the statuses and the backend error codes from every shape ofetch/h3 use,
// and it never returns the raw message or statusMessage of an untrusted
// failure. The raw-key print (the F-H3 symptom) means a caller reaches a
// client-facing string without going through this function.

const translate = vi.fn((key: string) => `translated:${key}`)

function statusError(status: number, code: string, message?: string) {
  return { response: { status, _data: { error: { code, message } } } }
}

describe('mapAuthError', () => {
  it('maps the EMAIL_ALREADY_EXISTS code through the same spelling as its key', () => {
    // The key lives under `errors` (plural): the call sites, the backend and
    // the four locales must all agree or the user sees the raw key again.
    expect(
      mapAuthError(statusError(422, 'EMAIL_ALREADY_EXISTS'), translate),
    ).toBe('translated:auth.register.errors.emailExists')
  })

  it('maps the WEAK_PASSWORD code through the same spelling as its key', () => {
    expect(
      mapAuthError(statusError(422, 'WEAK_PASSWORD'), translate),
    ).toBe('translated:auth.register.errors.weakPassword')
  })

  it('reads the code from the `data` shape too', () => {
    expect(
      mapAuthError(
        { data: { error: { code: 'EMAIL_ALREADY_EXISTS' } }, status: 409 },
        translate,
      ),
    ).toBe('translated:auth.register.errors.emailExists')
  })

  it('maps INVALID_CREDENTIALS on a login attempt', () => {
    expect(
      mapAuthError(statusError(401, 'INVALID_CREDENTIALS'), translate),
    ).toBe('translated:auth.login.error.invalidCredentials')
  })

  it('maps TOO_MANY_REQUESTS and RATE_LIMITED to the rate-limit message', () => {
    expect(mapAuthError(statusError(429, 'TOO_MANY_REQUESTS'), translate)).toBe(
      'translated:common.errorExtended.rateLimited',
    )
    expect(mapAuthError(statusError(429, 'RATE_LIMITED'), translate)).toBe(
      'translated:common.errorExtended.rateLimited',
    )
  })

  it('never returns a raw internal message for a 5xx', () => {
    expect(
      mapAuthError(
        statusError(500, 'INTERNAL', '[POST] "https://api.internal.example.com/api/v1/auth/login": fetch failed'),
        translate,
      ),
    ).toBe('translated:common.errorExtended.network')
  })

  it('returns the network message for a failure without any status', () => {
    expect(
      mapAuthError(new TypeError('fetch failed'), translate),
    ).toBe('translated:common.errorExtended.network')
  })

  it('returns the network message for a failure without any status', () => {
    expect(mapAuthError(new Error('something odd'), translate)).toBe(
      'translated:common.errorExtended.network',
    )
  })

  it('falls back to the caller-provided fallback key on shapes without a matching rule', () => {
    // A 3xx is neither a 4xx rejection, a 5xx, nor a network level: the
    // helper refuses to invent a user-facing story for it.
    expect(mapAuthError({ status: 302 }, translate, 'custom.fallback')).toBe(
      'translated:custom.fallback',
    )
  })
})
