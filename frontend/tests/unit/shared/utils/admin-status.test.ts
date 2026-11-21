import { describe, expect, it } from 'vitest'

import {
  formatSemanticStatusLabel,
  isSemanticStatusColumn,
  resolveSemanticStatusMeta,
} from '@/app/utils/admin-status'

describe('admin-status', () => {
  it('recognizes semantic status columns', () => {
    expect(isSemanticStatusColumn('status')).toBe(true)
    expect(isSemanticStatusColumn('payment_status')).toBe(true)
    expect(isSemanticStatusColumn('name')).toBe(false)
  })

  it('resolves a known status code', () => {
    const meta = resolveSemanticStatusMeta(1)
    expect(meta.key).toBe('active')
    expect(meta.badgeClass).toBe('badge-success')
  })

  it('falls back to unknown for an unrecognized code', () => {
    const meta = resolveSemanticStatusMeta(999)
    expect(meta.key).toBe('unknown')
  })

  it('formats a known label through the translator', () => {
    const t = (key: string) => (key === 'admin.statusLabels.active' ? 'Ativo' : key)
    expect(formatSemanticStatusLabel(1, t)).toBe('Ativo')
  })

  it('appends the raw code for an unknown numeric status', () => {
    const t = (key: string) => key
    expect(formatSemanticStatusLabel(42, t)).toBe('Unknown (42)')
  })
})
