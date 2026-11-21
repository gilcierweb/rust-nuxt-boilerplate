import {
  DATE_FORMAT_PRESETS,
  formatDate as formatDateUtil,
  formatDateLong as formatDateLongUtil,
  formatDateMedium as formatDateMediumUtil,
  formatDateShort as formatDateShortUtil,
  formatDateTime as formatDateTimeUtil,
  formatTime as formatTimeUtil,
  formatUtcDateShort as formatUtcDateShortUtil,
  parseDateValue,
  toDateInputValue as toDateInputValueUtil,
  toDateTimeInputValue as toDateTimeInputValueUtil,
  type DateFormatPreset,
  type DateInput,
} from '~/utils/format-date'

/**
 * System-wide formatting utilities for the frontend (reactive to app locale).
 * Date helpers use the shared `app/utils/format-date` utilities.
 */
export function useFormatters() {
  const { t, locale } = useI18n()

  function resolveEmptyValue(value: DateInput): string | null {
    if (value === null || value === undefined || value === '') {
      return t('admin.common.emptyValue')
    }
    return null
  }

  /** Format cents to the active locale. */
  function formatCents(cents: number, currency = false, currencyCode = 'BRL'): string {
    if (currency) {
      return (cents / 100).toLocaleString(locale.value, {
        style: 'currency',
        currency: currencyCode,
      })
    }
    return (cents / 100).toLocaleString(locale.value, {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
  }

  /** Format large numbers with k/M suffix. */
  function formatCount(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
    return String(n)
  }

  /** Format video duration in seconds to MM:SS. */
  function formatDuration(seconds: number): string {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${String(s).padStart(2, '0')}`
  }

  /** Format file size in bytes to human-readable. */
  function formatFileSize(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`
    return `${(bytes / 1024 ** 3).toFixed(2)} GB`
  }

  /** Format ISO date to relative time using i18n keys. */
  function timeAgo(dateStr: string): string {
    const empty = resolveEmptyValue(dateStr)
    if (empty !== null) return empty

    const date = parseDateValue(dateStr)
    if (!date) return String(dateStr)

    const diff = Date.now() - date.getTime()
    const mins = Math.floor(diff / 60_000)
    if (mins < 1) return t('timeAgo.now')
    if (mins < 60) return `${mins}${t('timeAgo.minutes')}`
    const hours = Math.floor(mins / 60)
    if (hours < 24) return `${hours}${t('timeAgo.hours')}`
    const days = Math.floor(hours / 24)
    if (days < 7) return `${days}${t('timeAgo.days')}`
    if (days < 30) return `${Math.floor(days / 7)}${t('timeAgo.weeks')}`

    return formatDateMediumUtil(dateStr, { locale: locale.value })
  }

  /**
   * Format a date with Intl options in the active locale.
   * Empty input returns the translated empty value for UI consistency.
   */
  function formatDate(
    dateStr: DateInput,
    options: Intl.DateTimeFormatOptions = { ...DATE_FORMAT_PRESETS.dateMedium },
  ): string {
    const empty = resolveEmptyValue(dateStr)
    if (empty !== null) return empty

    const date = parseDateValue(dateStr)
    if (!date) return String(dateStr)

    return new Intl.DateTimeFormat(locale.value, options).format(date)
  }

  /** Format a date/time with Intl options in the active locale. */
  function formatDateTime(
    dateStr: DateInput,
    options: Intl.DateTimeFormatOptions = { ...DATE_FORMAT_PRESETS.dateTime },
  ): string {
    const empty = resolveEmptyValue(dateStr)
    if (empty !== null) return empty

    const date = parseDateValue(dateStr)
    if (!date) return String(dateStr)

    return new Intl.DateTimeFormat(locale.value, options).format(date)
  }

  /** Format a time with Intl options in the active locale. */
  function formatTime(
    dateStr: DateInput,
    options: Intl.DateTimeFormatOptions = { ...DATE_FORMAT_PRESETS.time },
  ): string {
    const empty = resolveEmptyValue(dateStr)
    if (empty !== null) return empty

    const date = parseDateValue(dateStr)
    if (!date) return String(dateStr)

    return new Intl.DateTimeFormat(locale.value, options).format(date)
  }

  function formatByPreset(dateStr: DateInput, preset: DateFormatPreset): string {
    const empty = resolveEmptyValue(dateStr)
    if (empty !== null) return empty
    return formatDateUtil(dateStr, { locale: locale.value, preset })
  }

  /** Truncate text to maxLen characters. */
  function truncate(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text
    return `${text.slice(0, Math.max(0, maxLen - 3))}\u2026`
  }

  return {
    formatCents,
    formatCount,
    formatDuration,
    formatFileSize,
    timeAgo,
    formatDate,
    formatDateTime,
    formatTime,
    formatDateShort: (value: DateInput) => formatByPreset(value, 'dateShort'),
    formatDateMedium: (value: DateInput) => formatByPreset(value, 'dateMedium'),
    formatDateLong: (value: DateInput) => formatByPreset(value, 'dateLong'),
    formatDateFull: (value: DateInput) => formatByPreset(value, 'dateFull'),
    formatUtcDateShort: (value: unknown) => formatUtcDateShortUtil(value, { locale: locale.value }),
    toDateInputValue: toDateInputValueUtil,
    toDateTimeInputValue: toDateTimeInputValueUtil,
    truncate,
  }
}
