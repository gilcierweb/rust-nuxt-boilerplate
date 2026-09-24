import { describe, it, expect } from 'vitest'

import {
  PUBLIC_AUTH_PATHS,
  normalizeAuthPath,
  isAuthPath,
  isPublicAuthPath,
} from '../../../../shared/utils/auth-paths'

// ---------------------------------------------------------------------------
// normalizeAuthPath
// ---------------------------------------------------------------------------

describe('normalizeAuthPath', () => {
  it('keeps app-relative paths', () => {
    expect(normalizeAuthPath('/auth/login')).toBe('/auth/login')
  })

  it('adds a leading slash to proxy-relative paths', () => {
    expect(normalizeAuthPath('auth/login')).toBe('/auth/login')
  })

  it('strips the /api/v1 proxy prefix', () => {
    expect(normalizeAuthPath('/api/v1/auth/login')).toBe('/auth/login')
  })

  it('strips scheme, host, query and hash from full URLs', () => {
    expect(normalizeAuthPath('https://backend.example.com/api/v1/auth/login?x=1#y')).toBe(
      '/auth/login',
    )
  })

  it('strips trailing slashes and lowercases', () => {
    expect(normalizeAuthPath('/API/V1/Auth/Login/')).toBe('/auth/login')
  })

  it('returns empty string for non-string input', () => {
    expect(normalizeAuthPath(undefined)).toBe('')
    expect(normalizeAuthPath(null)).toBe('')
    expect(normalizeAuthPath(42)).toBe('')
    expect(normalizeAuthPath({ url: '/auth/login' })).toBe('/auth/login')
  })
})

// ---------------------------------------------------------------------------
// isAuthPath
// ---------------------------------------------------------------------------

describe('isAuthPath', () => {
  it('matches every /auth/** endpoint', () => {
    expect(isAuthPath('/auth/login')).toBe(true)
    expect(isAuthPath('/auth/refresh')).toBe(true)
    expect(isAuthPath('/auth/session')).toBe(true)
    expect(isAuthPath('/auth/logout')).toBe(true)
    expect(isAuthPath('/auth/magic-link/verify')).toBe(true)
  })

  it('matches versioned and absolute forms', () => {
    expect(isAuthPath('/api/v1/auth/refresh')).toBe(true)
    expect(isAuthPath('https://backend.example.com/api/v1/auth/session')).toBe(true)
  })

  it('does not match unrelated routes', () => {
    expect(isAuthPath('/admin/users')).toBe(false)
    expect(isAuthPath('/users/me')).toBe(false)
    // Segment-aware: prefixes without a boundary do not match.
    expect(isAuthPath('/auth-history')).toBe(false)
    expect(isAuthPath('/api/v1/authenticator/start')).toBe(false)
  })

  it('returns false for empty input', () => {
    expect(isAuthPath('')).toBe(false)
    expect(isAuthPath(undefined)).toBe(false)
  })
})

// ---------------------------------------------------------------------------
// isPublicAuthPath
// ---------------------------------------------------------------------------

describe('isPublicAuthPath', () => {
  it('matches every entry of PUBLIC_AUTH_PATHS', () => {
    for (const path of PUBLIC_AUTH_PATHS) {
      expect(isPublicAuthPath(path)).toBe(true)
    }
  })

  it('matches proxy-relative form (no leading slash)', () => {
    expect(isPublicAuthPath('auth/login')).toBe(true)
    expect(isPublicAuthPath('auth/logout')).toBe(true)
  })

  it('does not require a token-bearing round-trip for non-public auth paths', () => {
    expect(isPublicAuthPath('/auth/refresh')).toBe(false)
    expect(isPublicAuthPath('/auth/session')).toBe(false)
  })

  it('does not match unrelated routes', () => {
    expect(isPublicAuthPath('/admin/roles')).toBe(false)
    expect(isPublicAuthPath('/auth-history')).toBe(false)
  })
})
