import { describe, it, expect } from 'vitest'

import {
  NOTICE_KEYS,
  noticeKeyFromQuery,
  noticeTarget,
} from '../../../app/utils/flash-notice'

// ---------------------------------------------------------------------------
// flash-notice — a notice that survives a redirect, even an SSR one
// ---------------------------------------------------------------------------

describe('noticeKeyFromQuery', () => {
  it('names the denial key for the forbidden notice', () => {
    expect(noticeKeyFromQuery({ notice: 'forbidden' })).toBe('authorization.denied')
  })

  it('answers null for a notice it does not know', () => {
    expect(noticeKeyFromQuery({ notice: 'something-else' })).toBeNull()
  })

  it('answers null when the query carries no notice', () => {
    expect(noticeKeyFromQuery({})).toBeNull()
    expect(noticeKeyFromQuery({ notice: undefined })).toBeNull()
  })
})

describe('noticeTarget', () => {
  it('appends the notice to a plain path', () => {
    expect(noticeTarget('forbidden', '/portal')).toBe('/portal?notice=forbidden')
  })

  it('appends the notice to a localized path', () => {
    expect(noticeTarget('forbidden', '/pt-BR/portal')).toBe('/pt-BR/portal?notice=forbidden')
  })
})

describe('NOTICE_KEYS', () => {
  it('only holds notices with a locale key', () => {
    for (const key of Object.values(NOTICE_KEYS)) {
      expect(key).toMatch(/^[a-z]+\.[a-zA-Z]+$/)
    }
  })
})
