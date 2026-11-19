import { describe, expect, it } from 'vitest'

import { resolveApiBase } from '~/utils/api-base'

const configured = {
  public: {
    apiBase: '/api/v1',
    apiDirectBase: 'https://backend.example.com/api/v1/',
  },
}

describe('the api base of the current context', () => {
  it('renders the server through the proxy even when a direct base is configured', () => {
    // The server render carries the cookies of the browser through the Nitro
    // proxy, and the proxy attaches the access token it resolves from the
    // refresh cookie. A direct call from the SSR process drops both, so the
    // server keeps the proxy base whatever the client is configured to use.
    expect(resolveApiBase(configured, true)).toBe('/api/v1')
  });

  it('lets the client use the direct base when one is configured', () => {
    expect(resolveApiBase(configured, false)).toBe(
      'https://backend.example.com/api/v1',
    )
  })

  it('falls back to the proxy base on the client when no direct base is set', () => {
    expect(
      resolveApiBase({ public: { apiBase: '/api/v1', apiDirectBase: '' } }, false),
    ).toBe('/api/v1')
  })

  it('falls back to the proxy when the config carries no base at all', () => {
    expect(resolveApiBase({}, false)).toBe('/api/v1')
    expect(resolveApiBase({ public: undefined }, false)).toBe('/api/v1')
  })
})