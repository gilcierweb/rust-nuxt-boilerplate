/**
 * Shared formatting utilities used across components and pages.
 * All functions are pure (no side effects).
 */
export function useFormatters() {
  /** Format cents to BRL display string, e.g. 1990 → "19,90" */
  function formatCents(cents: number, currency = false): string {
    const val = (cents / 100).toLocaleString('pt-BR', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    })
    return currency ? `R$ ${val}` : val
  }

  /** Format large numbers with k/M suffix */
  function formatCount(n: number): string {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}k`
    return String(n)
  }

  /** Format video duration in seconds to MM:SS */
  function formatDuration(seconds: number): string {
    const m = Math.floor(seconds / 60)
    const s = seconds % 60
    return `${m}:${String(s).padStart(2, '0')}`
  }

  /** Format file size in bytes to human-readable */
  function formatFileSize(bytes: number): string {
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`
    return `${(bytes / 1024 ** 3).toFixed(2)} GB`
  }

  /** Format ISO date to relative time using i18n keys */
  function timeAgo(dateStr: string): string {
    const { t } = useI18n()
    const diff = Date.now() - new Date(dateStr).getTime()
    const mins = Math.floor(diff / 60_000)
    if (mins < 1) return t('timeAgo.now')
    if (mins < 60) return `${mins}${t('timeAgo.minutes')}`
    const hours = Math.floor(mins / 60)
    if (hours < 24) return `${hours}${t('timeAgo.hours')}`
    const days = Math.floor(hours / 24)
    if (days < 7) return `${days}${t('timeAgo.days')}`
    if (days < 30) return `${Math.floor(days / 7)}${t('timeAgo.weeks')}`
    return formatDate(dateStr, { month: 'short', day: '2-digit' })
  }

  /** Format ISO date to locale string */
  function formatDate(
    dateStr: string,
    options: Intl.DateTimeFormatOptions = { day: '2-digit', month: 'short', year: 'numeric' },
  ): string {
    if (!dateStr) return '—'
    return new Date(dateStr).toLocaleDateString('pt-BR', options)
  }

  /** Format ISO date to time string */
  function formatTime(dateStr: string): string {
    return new Date(dateStr).toLocaleTimeString('pt-BR', {
      hour: '2-digit',
      minute: '2-digit',
    })
  }

  /** Truncate text to maxLen characters */
  function truncate(text: string, maxLen: number): string {
    if (text.length <= maxLen) return text
    return text.slice(0, maxLen - 3) + '…'
  }

  return {
    formatCents,
    formatCount,
    formatDuration,
    formatFileSize,
    timeAgo,
    formatDate,
    formatTime,
    truncate,
  }
}