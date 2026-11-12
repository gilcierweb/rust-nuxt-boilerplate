import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import AdminPermissionMatrix from '@/app/components/admin/permissions/AdminPermissionMatrix.vue'
import type { Permission } from '@/app/types/rbac'

function permissionWith(
  code: string,
  links: { role_id: string; effect: 0 | 1 }[],
): Permission {
  return {
    id: code,
    code,
    name: code,
    description: null,
    resource: code.split(':')[0]!,
    action: code.split(':')[1]!,
    roles: links.map((link) => ({
      role_id: link.role_id,
      role_name: link.role_id,
      effect: link.effect,
    })),
  }
}

// The columns are keyed by role id: the unique index of the catalog is
// (name, scope), so two scoped roles may share a name and only the id tells
// the columns apart.
const adminRole = { id: 'role-admin', name: 'admin' }
const scopedAdminRole = { id: 'role-admin-scoped', name: 'admin' }

describe('AdminPermissionMatrix', () => {
  it('renders one row per permission and one column per role', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: [adminRole, { id: 'role-manager', name: 'manager' }],
        permissions: [
          permissionWith('users:read', [{ role_id: 'role-admin', effect: 0 }]),
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

  it('keeps two roles of the same name in separate columns', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: [adminRole, scopedAdminRole],
        permissions: [
          permissionWith('users:read', [{ role_id: 'role-admin-scoped', effect: 0 }]),
        ],
      },
    })

    const headers = wrapper.findAll('thead th')
    expect(headers).toHaveLength(3)
    expect(headers[2]!.text()).toBe('admin')

    // The allow badge answers for the scoped role, not for the class level one.
    expect(
      wrapper
        .find('[data-testid="cell-users:read-role-admin-scoped"]')
        .attributes('data-effect'),
    ).toBe('allow')
    expect(
      wrapper
        .find('[data-testid="cell-users:read-role-admin"]')
        .attributes('data-effect'),
    ).toBe('none')
  })

  it('renders an allow badge for effect 0', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: [adminRole],
        permissions: [permissionWith('users:read', [{ role_id: 'role-admin', effect: 0 }])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-role-admin"]')
    expect(cell.attributes('data-effect')).toBe('allow')
    expect(cell.text()).toBe('admin.permissions.effect.allow')
  })

  it('renders a deny badge for effect 1', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: [{ id: 'role-manager', name: 'manager' }],
        permissions: [permissionWith('users:read', [{ role_id: 'role-manager', effect: 1 }])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-role-manager"]')
    expect(cell.attributes('data-effect')).toBe('deny')
    expect(cell.text()).toBe('admin.permissions.effect.deny')
  })

  it('renders the none state when the role does not carry the permission', () => {
    const wrapper = mount(AdminPermissionMatrix, {
      props: {
        roles: [adminRole],
        permissions: [permissionWith('users:read', [])],
      },
    })

    const cell = wrapper.find('[data-testid="cell-users:read-role-admin"]')
    expect(cell.attributes('data-effect')).toBe('none')
    expect(cell.text()).toBe('admin.permissions.effect.none')
  })
})
