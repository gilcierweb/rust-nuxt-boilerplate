import { h, type VNode } from 'vue'

import { resolveDateLocale } from '~/utils/format-date'

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

  return new Intl.NumberFormat(resolveDateLocale(), {
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