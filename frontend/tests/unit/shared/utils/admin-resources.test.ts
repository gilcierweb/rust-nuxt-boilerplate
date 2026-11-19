import { describe, expect, it } from 'vitest'

import { toAuditDashboardPayload } from '~/utils/admin-resources'

const envelope = {
  data: [
    {
      id: 'log-1',
      action: 'user.created',
      resource_type: 'user',
      resource_id: 'user-1',
      actor_user_id: 'admin-1',
      ip_address: '10.0.0.1',
      user_agent: 'Mozilla/5.0',
      request_id: 'req-1',
      changes: { before: null, after: { email: 'a@b.c' } },
      metadata: { source: 'signup' },
      created_at: '2026-01-01T00:00:00Z',
    },
  ],
  pagination: { page: 1, per_page: 100, total: 9 },
}

describe('the audit payload the dashboard keeps', () => {
  it('trims every row to the fields the dashboard renders', () => {
    expect(toAuditDashboardPayload(envelope)).toEqual({
      data: [
        {
          id: 'log-1',
          action: 'user.created',
          resource_type: 'user',
          created_at: '2026-01-01T00:00:00Z',
        },
      ],
      pagination: { page: 1, per_page: 100, total: 9 },
    })
  })

  it('keeps the totals, which the charts read from the pagination meta', () => {
    expect(toAuditDashboardPayload(envelope).pagination.total).toBe(9)
  })

  it('leaves a payload that is not the list envelope untouched', () => {
    expect(toAuditDashboardPayload([])).toEqual([])
    expect(toAuditDashboardPayload(null)).toBe(null)
    expect(toAuditDashboardPayload({ data: null, pagination: null })).toEqual({
      data: null,
      pagination: null,
    })
  })
})