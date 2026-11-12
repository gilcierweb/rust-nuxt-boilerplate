import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import AdminPermissionMatrix from '@/app/components/admin/permissions/AdminPermissionMatrix.vue'
import type { Permission } from '@/app/types/rbac'

function permissionWith(
  code: string,
  links: { role_name: string; effect: 0 | 1 }[],
): Permission {
  return {
    id: code,
    code,
    name: code,
    description: null,
    resource: code.split(':')[0]!,
    action: code.split(':')[1]!,
    roles: links.map((link) => ({
      role_id: link.role_name,
      role_name: link.role_name,
      effect: link.effect,
    })),
    updated_at: '2026-01-01T00:00:00Z',
  }
}

describe('AdminPermissionMatrix', () => {
  it('renders one row per permission and one column per role', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: ['admin', 'manager'],
        permissions: [
          permissionWith('users:read', [{ role_name: 'admin', effect: 0 }]),
          permissionWith('users:update', []),
        ],
      },
    })

    const rows = wrapper.findAll('tbody tr')
    expect(rows).toHaveLength(2)
    expect(wrapper.text()).toContain('users:read')
    expect(wrapper.text()).toContain('users:update')

    const headers = wrapper.findAll('thead th')
    // The first header names the permission, the rest are the roles.
    expect(headers).toHaveLength(3)
    expect(headers[1]!.text()).toBe('admin')
    expect(headers[2]!.text()).toBe('manager')
  })

  it('renders an allow badge for effect 0', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: ['admin'],
        permissions: [permissionWith('users:read', [{ role_name: 'admin', effect: 0 }])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-admin"]')
    expect(cell.attributes('data-effect')).toBe('allow')
    expect(cell.text()).toBe('admin.permissions.effect.allow')
  })

  it('renders a deny badge for effect 1', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: ['manager'],
        permissions: [permissionWith('users:read', [{ role_name: 'manager', effect: 1 }])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-manager"]')
    expect(cell.attributes('data-effect')).toBe('deny')
    expect(cell.text()).toBe('admin.permissions.effect.deny')
  })

  it('renders the none state when the role does not carry the permission', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: ['admin'],
        permissions: [permissionWith('users:read', [])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-admin"]')
    expect(cell.attributes('data-effect')).toBe('none')
    expect(cell.text()).toBe('admin.permissions.effect.none')
  })
})
