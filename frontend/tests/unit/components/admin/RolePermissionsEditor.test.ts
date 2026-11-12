import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A mounted editor needs the real reactivity of the segmented control.
vi.unmock('vue')

import RolePermissionsEditor from '@/app/components/admin/roles/RolePermissionsEditor.vue'
import type { Permission, PermissionAssignment } from '@/app/types/rbac'

function permission(code: string, name: string): Permission {
  const [resource, action] = code.split(':')
  return {
    id: code,
    code,
    name,
    description: null,
    resource: resource!,
    action: action!,
    roles: [],
    updated_at: '2026-01-01T00:00:00Z',
  }
}

const catalog: Permission[] = [
  permission('roles:read', 'Read roles'),
  permission('roles:update', 'Update roles'),
  permission('users:read', 'Read users'),
]

const adminLinks: PermissionAssignment[] = [
  { code: 'roles:read', effect: 0 },
  { code: 'roles:update', effect: 0 },
  { code: 'users:read', effect: 0 },
]

function mountEditor(overrides: Record<string, unknown> = {}) {
  return mount(RolePermissionsEditor, {
    props: {
      permissions: catalog,
      assignments: adminLinks,
      ...overrides,
    },
    global: {
      stubs: {
        NuxtLink: true,
      },
    },
  })
}

describe('RolePermissionsEditor', () => {
  it('renders one row per permission grouped by resource', () => {
    const wrapper = mountEditor()

    const groups = wrapper.findAll('[data-testid="resource-group"]')
    expect(groups).toHaveLength(2)
    expect(groups[0]!.text()).toContain('roles')
    expect(groups[1]!.text()).toContain('users')
    expect(wrapper.findAll('[data-testid^="cell-"]')).toHaveLength(3)
  })

  it('seeds the cells from the loaded assignments', () => {
    const wrapper = mountEditor()

    expect(
      wrapper.find('[data-testid="cell-roles:read"]').attributes('data-value'),
    ).toBe('allow')
  })

  it('sends the deny effect and omits a permission left at none', async () => {
    const wrapper = mountEditor()

    await wrapper
      .find('[data-testid="cell-users:read"] [data-testid="option-deny"]')
      .trigger('click')
    await wrapper.find('form').trigger('submit')

    const payload = wrapper.emitted('save')?.[0]?.[0] as PermissionAssignment[]
    expect(payload).toEqual(
      expect.arrayContaining([
        { code: 'roles:read', effect: 0 },
        { code: 'roles:update', effect: 0 },
        { code: 'users:read', effect: 1 },
      ]),
    )
    expect(payload.find((entry) => entry.code === 'users:read')?.effect).toBe(1)
  })

  it('omits a permission moved to none', async () => {
    const wrapper = mountEditor()

    await wrapper
      .find('[data-testid="cell-users:read"] [data-testid="option-none"]')
      .trigger('click')
    await wrapper.find('form').trigger('submit')

    const payload = wrapper.emitted('save')?.[0]?.[0] as PermissionAssignment[]
    expect(payload.find((entry) => entry.code === 'users:read')).toBeUndefined()
  })

  it('warns when the edit would drop an administrator code', async () => {
    const wrapper = mountEditor()

    expect(wrapper.find('[data-testid="lockout-warning"]').exists()).toBe(false)

    await wrapper
      .find('[data-testid="cell-roles:read"] [data-testid="option-none"]')
      .trigger('click')

    expect(wrapper.find('[data-testid="lockout-warning"]').exists()).toBe(true)
  })

  // A deny link takes the authority away as surely as a removed one: deny beats
  // allow in the engine, so the warning has to read the effect, not the
  // presence of the code in the payload.
  it('warns when the edit would deny an administrator code', async () => {
    const wrapper = mountEditor()

    expect(wrapper.find('[data-testid="lockout-warning"]').exists()).toBe(false)

    await wrapper
      .find('[data-testid="cell-roles:read"] [data-testid="option-deny"]')
      .trigger('click')

    expect(wrapper.find('[data-testid="lockout-warning"]').exists()).toBe(true)
  })
})
