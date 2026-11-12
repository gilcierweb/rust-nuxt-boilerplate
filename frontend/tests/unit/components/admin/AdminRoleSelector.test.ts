import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A mounted selector needs the real reactivity of the checkboxes.
vi.unmock('vue')

import AdminRoleSelector from '@/app/components/admin/AdminRoleSelector.vue'
import type { Role } from '@/app/types/rbac'

const roles: Role[] = [
  { id: 'role-admin', name: 'admin', resource_type: null, resource_id: null, created_at: '', updated_at: '' },
  { id: 'role-manager', name: 'manager', resource_type: null, resource_id: null, created_at: '', updated_at: '' },
  { id: 'role-viewer', name: 'viewer', resource_type: null, resource_id: null, created_at: '', updated_at: '' },
]

function mountSelector(overrides: Record<string, unknown> = {}) {
  return mount(AdminRoleSelector, {
    props: {
      roles,
      selectedIds: ['role-manager'],
      ...overrides,
    },
    global: { stubs: { NuxtLink: true } },
  })
}

describe('AdminRoleSelector', () => {
  it('renders one checkbox per available role with the checked ones marked', () => {
    const wrapper = mountSelector()

    expect(wrapper.findAll('input[type="checkbox"]')).toHaveLength(3)
    const manager = wrapper.find('input[value="role-manager"]')
    expect((manager.element as HTMLInputElement).checked).toBe(true)
    expect((wrapper.find('input[value="role-viewer"]').element as HTMLInputElement).checked).toBe(
      false,
    )
  })

  it('sends the checked role ids on save', async () => {
    const wrapper = mountSelector()

    await wrapper.find('input[value="role-viewer"]').setValue(true)
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('save')).toEqual([[['role-manager', 'role-viewer']]])
  })

  it('sends the reduced list when a role is unchecked', async () => {
    const wrapper = mountSelector()

    await wrapper.find('input[value="role-manager"]').setValue(false)
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('save')).toEqual([[[]]])
  })

  it('surfaces the server message of a refused save', () => {
    const wrapper = mountSelector({
      errorMessage: 'A operacao exige autoridades que o ator nao possui.',
    })

    const alert = wrapper.find('[data-testid="role-selector-error"]')
    expect(alert.exists()).toBe(true)
    expect(alert.text()).toContain('autoridades')
  })

  it('blocks the submit while saving', () => {
    const wrapper = mountSelector({ saving: true })

    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeDefined()
  })
})
