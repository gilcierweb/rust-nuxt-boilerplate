/**
 * System-wide date formatting utilities (Nuxt auto-imported from app/utils).
 *
 * - Pure functions: safe in SSR, client, and non-Vue contexts.
 * - Locale-aware: defaults to the active app locale via `useI18n()` when
 *   available, otherwise falls back to `pt-BR`.
 * - Timezone-aware: UTC presets are explicit for date-only/bucketed values
 *   (e.g. chart axes) to avoid day shifts. Local-time presets keep JS defaults
 *   unless `timeZone` is provided.
 * - Backward-compatible: `formatDate`/`formatDateTime` accept `(value, locale)`
 *   as a legacy second argument.
 */

export type DateInput = string | number | Date | null | undefined

export type DateFormatPresetName =
  | 'date'
  | 'dateNumeric'
  | 'dateShortYear'
  | 'dateShort'
  | 'dateMedium'
  | 'dateLong'
  | 'dateFull'
  | 'dateTime'
  | 'dateTimeShort'
  | 'dateTimeLong'
  | 'dateTimeFull'
  | 'time'
  | 'timeWithSeconds'
  | 'monthYear'
  | 'monthYearNumeric'
  | 'utcDate'
  | 'utcDateShort'

export type DateFormatPresetAlias =
  | 'short'
  | 'curto'
  | 'medium'
  | 'medio'
  | 'long'
  | 'extenso'
  | 'full'
  | 'completo'
  | 'dateHour'
  | 'dataHora'
  | 'hour'
  | 'hora'
  | 'dayMonth'

export type DateFormatPreset = DateFormatPresetName | DateFormatPresetAlias

export interface FormatDateOptions extends Intl.DateTimeFormatOptions {
  locale?: string
  preset?: DateFormatPreset
}

const DEFAULT_LOCALE = 'pt-BR'

const BASE_PRESETS: Record<DateFormatPresetName, Intl.DateTimeFormatOptions> = {
  date: { day: '2-digit', month: '2-digit', year: 'numeric' },
  dateNumeric: { day: '2-digit', month: '2-digit', year: 'numeric' },
  dateShortYear: { day: '2-digit', month: '2-digit', year: '2-digit' },
  dateShort: { day: '2-digit', month: '2-digit' },
  dateMedium: { day: '2-digit', month: 'short', year: 'numeric' },
  dateLong: { day: '2-digit', month: 'long', year: 'numeric' },
  dateFull: { weekday: 'long', day: '2-digit', month: 'long', year: 'numeric' },
  dateTime: { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  dateTimeShort: { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' },
  dateTimeLong: { day: '2-digit', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' },
  dateTimeFull: {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  },
  time: { hour: '2-digit', minute: '2-digit' },
  timeWithSeconds: { hour: '2-digit', minute: '2-digit', second: '2-digit' },
  monthYear: { month: 'long', year: 'numeric' },
  monthYearNumeric: { month: '2-digit', year: 'numeric' },
  utcDate: { day: '2-digit', month: '2-digit', year: 'numeric', timeZone: 'UTC' },
  utcDateShort: { day: '2-digit', month: '2-digit', timeZone: 'UTC' },
}

const PRESET_ALIASES: Record<DateFormatPresetAlias, DateFormatPresetName> = {
  short: 'date',
  curto: 'date',
  medium: 'dateMedium',
  medio: 'dateMedium',
  long: 'dateLong',
  extenso: 'dateLong',
  full: 'dateFull',
  completo: 'dateFull',
  dateHour: 'dateTime',
  dataHora: 'dateTime',
  hour: 'time',
  hora: 'time',
  dayMonth: 'dateShort',
}

export const DATE_FORMAT_PRESETS: Record<DateFormatPresetName, Intl.DateTimeFormatOptions> = BASE_PRESETS

export function resolveDateLocale(fallback: string = DEFAULT_LOCALE): string {
  try {
    const { locale } = useI18n()
    const value = locale?.value
    return typeof value === 'string' && value.length > 0 ? value : fallback
  } catch {
    return fallback
  }
}

export function parseDateValue(value: DateInput): Date | null {
  if (value === null || value === undefined || value === '') return null

  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? null : value
  }

  if (typeof value === 'number') {
    const date = new Date(value)
    return Number.isNaN(date.getTime()) ? null : date
  }

  if (typeof value === 'string') {
    const date = new Date(value)
    return Number.isNaN(date.getTime()) ? null : date
  }

  return null
}

function isFormatOptions(value: unknown): value is FormatDateOptions {
  return typeof value === 'object' && value !== null && !(value instanceof Date)
}

function resolvePreset(preset?: DateFormatPreset): Intl.DateTimeFormatOptions {
  if (!preset) return BASE_PRESETS.date
  const name = (PRESET_ALIASES as Record<string, DateFormatPresetName>)[preset] ?? (preset as DateFormatPresetName)
  return BASE_PRESETS[name] ?? BASE_PRESETS.date
}

function normalizeFormatArgs(
  localeOrOptions: string | FormatDateOptions | undefined,
  defaultPreset: DateFormatPreset,
): { locale: string; options: Intl.DateTimeFormatOptions } {
  if (typeof localeOrOptions === 'string') {
    return { locale: localeOrOptions, options: { ...resolvePreset(defaultPreset) } }
  }

  const { locale, preset, ...intlOptions } = localeOrOptions ?? {}

  return {
    locale: locale ?? resolveDateLocale(),
    options: { ...resolvePreset(preset ?? defaultPreset), ...intlOptions },
  }
}

function formatWithIntl(date: Date, locale: string, options: Intl.DateTimeFormatOptions): string {
  return new Intl.DateTimeFormat(locale, options).format(date)
}

/**
 * Format a date-like value. Returns empty string for null/undefined/empty input.
 * For an unparseable string input, returns the original string (backward compat).
 */
export function formatDateWithPreset(
  value: DateInput,
  preset: DateFormatPreset = 'date',
  localeOrOptions?: string | FormatDateOptions,
): string {
  if (value === null || value === undefined || value === '') return ''

  let locale: string
  let options: Intl.DateTimeFormatOptions

  if (typeof localeOrOptions === 'string') {
    locale = localeOrOptions
    options = { ...resolvePreset(preset) }
  } else if (isFormatOptions(localeOrOptions)) {
    const { locale: optLocale, preset: ignoredPreset, ...intlOptions } = localeOrOptions
    locale = optLocale ?? resolveDateLocale()
    options = { ...resolvePreset(preset), ...intlOptions }
  } else {
    locale = resolveDateLocale()
    options = { ...resolvePreset(preset) }
  }

  const date = parseDateValue(value)
  if (!date) {
    // Invalid text keeps its content (legacy contract); other unexpected
    // shapes (arrays/objects from a malformed source) are not displayed raw.
    return typeof value === 'string' ? value : ''
  }

  return formatWithIntl(date, locale, options)
}

export function formatDate(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'date', localeOrOptions)
}

export function formatDateTime(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'dateTime', localeOrOptions)
}

export function formatTime(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'time', localeOrOptions)
}

export function formatDateShort(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'dateShort', localeOrOptions)
}

export function formatDateMedium(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'dateMedium', localeOrOptions)
}

export function formatDateLong(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'dateLong', localeOrOptions)
}

export function formatDateFull(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'dateFull', localeOrOptions)
}

export function formatUtcDate(value: DateInput, localeOrOptions?: string | FormatDateOptions): string {
  return formatDateWithPreset(value, 'utcDate', localeOrOptions)
}

/**
 * Format a UTC day key/value as a short day/month label (e.g. chart axes).
 * Robust to non-string tick values passed by chart libraries.
 */
export function formatUtcDateShort(value: unknown, localeOrOptions?: string | FormatDateOptions): string {
  let locale = resolveDateLocale()
  let options: Intl.DateTimeFormatOptions = { ...BASE_PRESETS.utcDateShort }

  if (typeof localeOrOptions === 'string') {
    locale = localeOrOptions
  } else if (isFormatOptions(localeOrOptions)) {
    const { locale: optLocale, preset, ...intlOptions } = localeOrOptions
    locale = optLocale ?? locale
    options = { ...resolvePreset(preset ?? 'utcDateShort'), ...intlOptions }
  }

  if (value instanceof Date) {
    return Number.isNaN(value.getTime()) ? '' : formatWithIntl(value, locale, options)
  }

  if (typeof value === 'number' && Number.isFinite(value)) {
    // Large values are treated as timestamps; small integers are likely indices.
    if (Math.abs(value) <= 1e10) return String(value)
    const date = new Date(value)
    return Number.isNaN(date.getTime()) ? '' : formatWithIntl(date, locale, options)
  }

  if (typeof value !== 'string' || value.length === 0) return ''

  const date = parseDateValue(value)
  if (!date) return value

  return formatWithIntl(date, locale, options)
}

export function toDateInputValue(value: DateInput): string {
  const date = parseDateValue(value)
  if (!date) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')

  return `${year}-${month}-${day}`
}

export function toDateTimeInputValue(value: DateInput): string {
  const date = parseDateValue(value)
  if (!date) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  const hours = `${date.getHours()}`.padStart(2, '0')
  const minutes = `${date.getMinutes()}`.padStart(2, '0')

  return `${year}-${month}-${day}T${hours}:${minutes}`
}
