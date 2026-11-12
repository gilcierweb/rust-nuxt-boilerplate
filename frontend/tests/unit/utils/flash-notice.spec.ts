import { describe, it, expect } from 'vitest'

import {
  NOTICE_KEYS,
  noticeKeyFromQuery,
  noticeQuery,
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

describe('noticeQuery', () => {
  it('carries a known notice so a forwarding redirect keeps it alive', () => {
    expect(noticeQuery({ notice: 'forbidden' })).toEqual({ notice: 'forbidden' })
  })

  it('answers undefined for a query with no notice', () => {
    expect(noticeQuery({})).toBeUndefined()
  })

  it('drops a notice it does not know instead of forwarding it', () => {
    expect(noticeQuery({ notice: 'something-else' })).toBeUndefined()
  })

  it('forwards the notice alone, never the rest of the query', () => {
    // A paginated link must not hand its page to the target of a redirect.
    expect(noticeQuery({ notice: 'forbidden', page: '2' })).toEqual({ notice: 'forbidden' })
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
