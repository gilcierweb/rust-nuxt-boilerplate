import { h, type VNode } from 'vue'

/**
 * The locale of the app, read from the i18n composer when the call happens
 * inside a component and from the browser otherwise. A fixed locale printed a
 * date in Portuguese while the interface was in English or Spanish.
 */
function resolveLocale(): string {
  try {
    const { locale } = useI18n()
    return locale.value ?? 'pt-BR'
  } catch {
    return 'pt-BR'
  }
}

// Format ISO datetime to the locale of the app. A missing value answers an
// empty string: the caller names the empty state through its own key, so no
// decorative character reaches the screen.
export function formatDateTime(value?: string | null, locale: string = resolveLocale()): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString(locale, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function formatDate(value?: string | null, locale: string = resolveLocale()): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleDateString(locale, {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
  })
}

export function toDateInputValue(value?: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')

  return `${year}-${month}-${day}`
}

export function toDateTimeInputValue(value?: string | null): string {
  if (!value) return ''
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return ''

  const year = date.getFullYear()
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  const hours = `${date.getHours()}`.padStart(2, '0')
  const minutes = `${date.getMinutes()}`.padStart(2, '0')

  return `${year}-${month}-${day}T${hours}:${minutes}`
}

export function stringifyJson(value: unknown): string {
  if (value === undefined || value === null || value === '') return '{}'
  if (typeof value === 'string') return value

  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return '{}'
  }
}

export function formatCurrency(
  value?: string | number | null,
  currencyCode: string = 'BRL',
): string {
  if (value === undefined || value === null || value === '') return ''

  const amount = typeof value === 'number' ? value : Number(String(value).replace(',', '.'))
  if (Number.isNaN(amount)) return String(value)

  return new Intl.NumberFormat(resolveLocale(), {
    style: 'currency',
    currency: currencyCode || 'BRL',
  }).format(amount)
}

// Row delete button for admin index pages (shared by every admin table).
// Native-element children MUST be an ARRAY of vnodes: a slot object
// ({ default: () => vnode }) returning a single VNode is silently discarded
// for native elements, because it is never normalized by the runtime (the vnode
// keeps the slot object with no ARRAY_CHILDREN flag and no dev warning), so the
// button renders empty and the icon line disappears.
export function adminDeleteButton(options: {
  pending: boolean
  onClick: () => void
}): VNode {
  return h(
    'button',
    {
      type: 'button',
      class: 'btn btn-circle btn-text btn-sm text-error',
      disabled: options.pending,
      onClick: options.onClick,
    },
    [
      options.pending
        ? h('span', { class: 'icon-[tabler--loader-2] size-5 animate-spin' })
        : h('span', { class: 'icon-[tabler--trash] size-5' }),
    ],
  )
}
