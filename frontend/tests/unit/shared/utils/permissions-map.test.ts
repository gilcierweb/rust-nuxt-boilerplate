import { describe, expect, it } from 'vitest'

import { toPermissionTableRow, toPermissionTableRows } from '@/app/utils/permissions-map'
import type { Permission } from '@/app/types/rbac'

const permission: Permission = {
  id: 'permission-1',
  code: 'users:read',
  name: 'Read users',
  description: 'Allows reading the user list.',
  resource: 'users',
  action: 'read',
  roles: [
    { role_id: 'role-admin', role_name: 'admin', effect: 0 },
    { role_id: 'role-manager', role_name: 'manager', effect: 1 },
  ],
  updated_at: '2026-01-02T03:04:05Z',
}

describe('toPermissionTableRow', () => {
  it('compiles the carrying role names in link order', () => {
    expect(toPermissionTableRow(permission).roles).toEqual(['admin', 'manager'])
  })

  it('keeps the identity and the descriptive fields', () => {
    const row = toPermissionTableRow(permission)
    expect(row.id).toBe('permission-1')
    expect(row.name).toBe('Read users')
    expect(row.code).toBe('users:read')
    expect(row.description).toBe('Allows reading the user list.')
    expect(row.resource).toBe('users')
    expect(row.action).toBe('read')
    expect(row.updated_at).toBe('2026-01-02T03:04:05Z')
  })

  it('answers an empty role list for a permission nobody carries', () => {
    const row = toPermissionTableRow({ ...permission, roles: [] })
    expect(row.roles).toEqual([])
  })

  it('maps a list without losing rows', () => {
    expect(toPermissionTableRows([permission, { ...permission, id: 'permission-2' }])).toHaveLength(
      2,
    )
  })
})
