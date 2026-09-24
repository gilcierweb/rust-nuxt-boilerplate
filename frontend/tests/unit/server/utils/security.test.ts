import { describe, it, expect } from 'vitest'

import { resolveApiDirectBase } from '../../../../server/utils/security'

// ---------------------------------------------------------------------------
// resolveApiDirectBase
// ---------------------------------------------------------------------------

describe('resolveApiDirectBase', () => {
  it('keeps an empty value (proxy default) untouched', () => {
    expect(
      resolveApiDirectBase({ apiDirectBase: '', isProdLike: true }),
    ).toEqual({ value: '', enforced: false })
  })

  it('enforces the proxy default when a direct base is set in prod', () => {
    expect(
      resolveApiDirectBase({
        apiDirectBase: 'https://backend.example.com/api/v1',
        isProdLike: true,
      }),
    ).toEqual({ value: '', enforced: true })
  })

  it('also enforces in staging', () => {
    // isProdLike already covers staging; exercised via the same flag.
    expect(
      resolveApiDirectBase({
        apiDirectBase: 'https://backend.example.com',
        isProdLike: true,
      }),
    ).toEqual({ value: '', enforced: true })
  })

  it('keeps the value when the operator explicitly opts out', () => {
    expect(
      resolveApiDirectBase({
        apiDirectBase: 'https://backend.example.com/api/v1',
        isProdLike: true,
        allowDirectApi: 'true',
      }),
    ).toEqual({
      value: 'https://backend.example.com/api/v1',
      enforced: false,
    })
  })

  it('does not enforce in non-production (dev warning path)', () => {
    expect(
      resolveApiDirectBase({
        apiDirectBase: 'http://localhost:8080/api/v1',
        isProdLike: false,
      }),
    ).toEqual({
      value: 'http://localhost:8080/api/v1',
      enforced: false,
    })
  })

  it('treats non-string values as unset', () => {
    expect(
      resolveApiDirectBase({ apiDirectBase: undefined, isProdLike: true }),
    ).toEqual({ value: '', enforced: false })
  })
})
