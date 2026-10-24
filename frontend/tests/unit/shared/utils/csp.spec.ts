import { describe, expect, it } from 'vitest'

import { contentSecurityPolicy } from '~~/shared/utils/csp'

/**
 * nuxt-security replaces the entire CSP object given to it, so anything the
 * app depends on must be listed here. These tests pin the directives that were
 * silently dropped when the policy was last overridden.
 */
describe('contentSecurityPolicy', () => {
  const policy = contentSecurityPolicy(false)

  it('restricts object, base and form targets', () => {
    expect(policy['object-src']).toContain("'none'")
    expect(policy['base-uri']).toContain("'none'")
    expect(policy['form-action']).toContain("'self'")
    expect(policy['frame-ancestors']).toContain("'self'")
  })

  it('does not allow inline script attributes', () => {
    expect(policy['script-src-attr']).toContain("'none'")
  })

  it('upgrades insecure requests in production', () => {
    expect(policy['upgrade-insecure-requests']).toBe(true)
  })

  it('scopes connect-src to the same origin in production', () => {
    expect(policy['connect-src']).toEqual(["'self'"])
  })

  it('allows local websocket origins only in development', () => {
    const dev = contentSecurityPolicy(true)
    expect(dev['connect-src']).toContain("'self'")
    expect(dev['connect-src']).toContain('ws://localhost:*')

    // The production policy must not widen the same directive.
    expect(policy['connect-src']).not.toContain('ws://localhost:*')
  })

  it('keeps the directives the app needs to boot', () => {
    expect(policy['default-src']).toContain("'self'")
    expect(policy['img-src']).toContain('data:')
    expect(policy['font-src']).toContain('fonts.gstatic.com')
    expect(policy['style-src']).toContain("'unsafe-inline'")
    expect(policy['script-src']).toContain("'self'")
  })
})
