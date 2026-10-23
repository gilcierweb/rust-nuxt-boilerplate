import { h, type VNode } from 'vue'

// Format ISO datetime to pt-BR locale string
export function formatDateTime(value?: string | null): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleString('pt-BR', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

export function formatDate(value?: string | null): string {
  if (!value) return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return String(value)
  return date.toLocaleDateString('pt-BR', {
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
  if (value === undefined || value === null || value === '') return '—'

  const amount = typeof value === 'number' ? value : Number(String(value).replace(',', '.'))
  if (Number.isNaN(amount)) return String(value)

  return new Intl.NumberFormat('pt-BR', {
    style: 'currency',
    currency: currencyCode || 'BRL',
  }).format(amount)
}

// Extract error message from API response
export function extractErrorMessage(error: any): string {
  return (
    error?.data?.error?.message ||
    error?.response?._data?.error?.message ||
    error?.statusMessage ||
    error?.message ||
    'Could not load data.'
  )
}

// Row delete button for admin index pages (shared by every admin table).
// Native-element children MUST be an ARRAY of vnodes: a slot object
// ({ default: () => vnode }) returning a single VNode is silently discarded
// for native elements — it is never normalized by the runtime (the vnode keeps
// the slot object with no ARRAY_CHILDREN flag and no dev warning), so the
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
