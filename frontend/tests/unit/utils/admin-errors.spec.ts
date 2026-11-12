import { describe, it, expect } from 'vitest'

import { mapAdminError } from '../../../app/utils/admin-errors'

// ---------------------------------------------------------------------------
// mapAdminError — safe localized messages for the admin data paths
// ---------------------------------------------------------------------------

// A translate function that answers the key itself, so an assertion reads the
// key the production code chose instead of a translated string.
const translate = (key: string) => `key:${key}`

describe('mapAdminError', () => {
  it('names the denial key for a FORBIDDEN code', () => {
    const error = {
      statusCode: 403,
      data: { error: { code: 'FORBIDDEN', message: 'Admin domain permissions required' } },
    }

    expect(mapAdminError(error, translate)).toBe('key:authorization.denied')
  })

  it('names the denial key when the code only rides the response body', () => {
    const error = {
      response: {
        status: 403,
        _data: { error: { code: 'FORBIDDEN', message: 'Admin domain permissions required' } },
      },
    }

    expect(mapAdminError(error, translate)).toBe('key:authorization.denied')
  })

  it('trusts a localized backend message of another 4xx', () => {
    // The backend localizes 4xx messages through Accept-Language, and the
    // message is authored by us: a validation refusal is shown as is.
    const error = {
      statusCode: 422,
      data: { error: { code: 'VALIDATION_ERROR', message: 'Um campo precisa de atenção' } },
    }

    expect(mapAdminError(error, translate)).toBe('Um campo precisa de atenção')
  })

  it('falls back to a key when a 4xx carries no message', () => {
    const error = { statusCode: 409, data: { error: { code: 'CONFLICT' } } }

    expect(mapAdminError(error, translate)).toBe('key:common.errorExtended.unexpected')
  })

  it('never leaks a 5xx or a network failure', () => {
    const serverError = {
      statusCode: 500,
      message: 'connect ECONNREFUSED 10.0.0.4:8080',
    }
    const networkError = { message: 'NetworkError when attempting to fetch resource.' }

    expect(mapAdminError(serverError, translate)).toBe('key:common.errorExtended.network')
    expect(mapAdminError(networkError, translate)).toBe('key:common.errorExtended.network')
  })

  it('names the rate limited key for the throttled requests', () => {
    const error = {
      statusCode: 429,
      data: { error: { code: 'RATE_LIMITED', message: '' } },
    }

    expect(mapAdminError(error, translate)).toBe('key:common.errorExtended.rateLimited')
  })

  it('honors a caller chosen fallback key', () => {
    expect(mapAdminError({ statusCode: 409 }, translate, 'admin.common.errorLoadingData')).toBe(
      'key:admin.common.errorLoadingData',
    )
  })
})
