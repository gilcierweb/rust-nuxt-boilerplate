import { describe, expect, it } from 'vitest'

import {
  authTokenFromQuery,
  pathWithoutAuthToken,
} from '~~/shared/utils/auth-token-url'

describe('authTokenFromQuery', () => {
  it('extracts the token from a query string', () => {
    expect(authTokenFromQuery({ token: 'abc123' })).toBe('abc123')
  })

  it('returns null when the parameter is absent', () => {
    expect(authTokenFromQuery({})).toBeNull()
    expect(authTokenFromQuery({ other: 'x' })).toBeNull()
  })

  it('returns null for an empty or non-string value', () => {
    expect(authTokenFromQuery({ token: '' })).toBeNull()
    expect(authTokenFromQuery({ token: ['a', 'b'] })).toBeNull()
    expect(authTokenFromQuery(undefined)).toBeNull()
  })
})

describe('pathWithoutAuthToken', () => {
  it('drops the token and keeps the rest of the query', () => {
    expect(pathWithoutAuthToken('/auth/reset-password', { token: 'abc', next: '/portal' })).toBe(
      '/auth/reset-password?next=%2Fportal',
    )
  })

  it('drops only the specified parameter', () => {
    expect(
      pathWithoutAuthToken('/auth/magic-link-verify', { token: 'abc', keep: '1' }, 'token'),
    ).toBe('/auth/magic-link-verify?keep=1')
  })

  it('returns the bare path when the token was the only parameter', () => {
    expect(pathWithoutAuthToken('/auth/reset-password', { token: 'abc' })).toBe(
      '/auth/reset-password',
    )
  })

  it('preserves repeated parameters', () => {
    expect(pathWithoutAuthToken('/p', { token: 'abc', tag: ['x', 'y'] })).toBe('/p?tag=x&tag=y')
  })
})
