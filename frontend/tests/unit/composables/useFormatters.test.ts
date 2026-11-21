import { describe, expect, it, vi } from 'vitest'
import { ref } from 'vue'

// The composable reads the locale of the running app: the shared setup mocks
// useI18n with the identity translation only, so this test provides the
// current locale the way the i18n module does at runtime. The bare identifier
// resolves from the global scope, the same way the app resolves it.
vi.unmock('vue')

const currentLocale = ref('en-US')
const globalScope = globalThis as any
globalScope.useI18n = () => ({
  t: (key: string) => key,
  locale: currentLocale,
})

const { useFormatters } = await import('~/composables/useFormatters')

describe('useFormatters', () => {
  it('formats dates in the locale of the app, not a fixed one', () => {
    const { formatDate } = useFormatters()
    const options = { day: '2-digit', month: 'short', year: 'numeric', timeZone: 'UTC' }
    // en-US prints "Nov 18, 2026"; pt-BR would print "18 de nov. de 2026".
    expect(formatDate('2026-11-18T12:00:00Z', options)).toBe('Nov 18, 2026')

    currentLocale.value = 'pt-BR'
    expect(formatDate('2026-11-18T12:00:00Z', options)).toBe('18 de nov. de 2026')
    currentLocale.value = 'en-US'
  })

  it('formats the empty date through the i18n key, never a literal em dash', () => {
    const { formatDate, timeAgo } = useFormatters()
    expect(formatDate('')).toBe('admin.common.emptyValue')
    expect(timeAgo('')).toBe('admin.common.emptyValue')
  })

  it('formats money through the locale currency format', () => {
    const { formatCents } = useFormatters()
    expect(formatCents(1990)).toBe('19.90')
    expect(formatCents(1990, true)).toContain('19.90')
  })

  it('formats relative time from the i18n keys', () => {
    const { timeAgo } = useFormatters()
    expect(timeAgo(new Date().toISOString())).toBe('timeAgo.now')

    const fiveMinutesAgo = new Date(Date.now() - 5 * 60_000).toISOString()
    expect(timeAgo(fiveMinutesAgo)).toBe('5timeAgo.minutes')
  })

  it('formats counts, durations and sizes without a locale', () => {
    const { formatCount, formatDuration, formatFileSize, truncate } = useFormatters()
    expect(formatCount(1500)).toBe('1.5k')
    expect(formatDuration(125)).toBe('2:05')
    expect(formatFileSize(2048)).toBe('2.0 KB')
    expect(truncate('abcdef', 4)).toBe('a…')
  })
})