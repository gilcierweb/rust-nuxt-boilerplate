import { describe, it, expect } from 'vitest'

import {
  formatDate,
  formatDateLong,
  formatDateTime,
  formatTime,
  formatUtcDateShort,
  parseDateValue,
  toDateInputValue,
  toDateTimeInputValue,
} from '@/app/utils/format-date'

describe('parseDateValue', () => {
  it('parses strings, numbers and Date instances', () => {
    expect(parseDateValue('2026-11-20T15:37:36Z')).toBeInstanceOf(Date)
    expect(parseDateValue(0)?.getTime()).toBe(0)
    expect(parseDateValue('not a date')).toBeNull()
    expect(parseDateValue(null)).toBeNull()
  })
})

describe('the date formatters', () => {
  it('answers an empty string for a missing value', () => {
    expect(formatDateTime(undefined)).toBe('')
    expect(formatDate(null)).toBe('')
    expect(formatTime('')).toBe('')
  })

  it('formats a readable instant in the locale of the caller', () => {
    expect(formatDateTime('2026-11-20T15:37:36Z', 'en-US')).toContain('2026')
    expect(formatDateTime('2026-11-20T15:37:36Z', 'pt-BR')).toContain('20/11/2026')
  })

  it('hands back an unparseable string unchanged', () => {
    expect(formatDateTime('not a date')).toBe('not a date')
  })

  it('formats with the presets of the system', () => {
    expect(formatDateLong('2026-11-20T15:37:36Z', { locale: 'en-US' })).toContain('2026')
    expect(formatUtcDateShort('2026-11-20', 'pt-BR')).toBe('20/11')
    expect(formatUtcDateShort('not a date', 'pt-BR')).toBe('not a date')
  })

  it('survives the malformed values a chart axis may pass', () => {
    expect(formatUtcDateShort([2026, 316, 15, 37, 36, 177671000, 0, 0, 0], 'pt-BR')).toBe('')
    expect(formatUtcDateShort({ created_at: '2026-11-20' }, 'pt-BR')).toBe('')
    expect(formatUtcDateShort(new Date('2026-11-20T00:00:00Z'), 'pt-BR')).toBe('20/11')
  })
})

describe('the date input helpers', () => {
  it('fills the html date and datetime-local inputs', () => {
    expect(toDateInputValue('2026-11-20T12:30:00')).toBe('2026-11-20')
    expect(toDateTimeInputValue('2026-11-20T12:30:00')).toBe('2026-11-20T12:30')
    expect(toDateInputValue('nope')).toBe('')
  })
})