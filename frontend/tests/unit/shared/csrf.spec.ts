import { describe, expect, it } from 'vitest'

import { csrfTokenFromCookieHeader } from '~~/shared/utils/csrf'

describe('csrfTokenFromCookieHeader', () => {
  it('extracts the csrf token from a single-cookie header', () => {
    expect(
      csrfTokenFromCookieHeader('csrf_token=1700000000.00112233.aabbccdd'),
    ).toBe('1700000000.00112233.aabbccdd')
  })

  it('extracts the csrf token among other cookies', () => {
    expect(
      csrfTokenFromCookieHeader(
        'refresh_token=abc; csrf_token=ts.nonce.sig; theme=dark',
      ),
    ).toBe('ts.nonce.sig')
  })

  it('parses cookie rows separated by ";" without spaces', () => {
    expect(csrfTokenFromCookieHeader('refresh_token=abc;csrf_token=x.y.z')).toBe('x.y.z')
  })

  it('returns null when the csrf cookie is absent', () => {
    expect(csrfTokenFromCookieHeader('refresh_token=abc; theme=dark')).toBeNull()
  })

  it('returns null for empty or missing input', () => {
    expect(csrfTokenFromCookieHeader('')).toBeNull()
    expect(csrfTokenFromCookieHeader(undefined)).toBeNull()
    expect(csrfTokenFromCookieHeader(null)).toBeNull()
  })

  it('preserves "=" characters inside the token value', () => {
    // Token format is `timestamp.nonce.hmac` (no "="), but the parser must
    // not truncate values that contain one.
    expect(csrfTokenFromCookieHeader('csrf_token=a=b')).toBe('a=b')
  })

  it('ignores similarly-named cookies', () => {
    expect(csrfTokenFromCookieHeader('csrf_token_legacy=old; other=x')).toBeNull()
  })
})
