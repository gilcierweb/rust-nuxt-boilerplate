import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A mounted form needs the real reactivity of v-model and computed.
vi.unmock('vue')

import PermissionsForm from '@/app/components/admin/permissions/PermissionsForm.vue'
import type { PermissionCatalog } from '@/app/types/rbac'

const catalog: PermissionCatalog = {
  resources: [
    { code: 'users', label_key: 'authorization.resources.users' },
    { code: 'upload', label_key: 'authorization.resources.upload' },
  ],
  actions: [
    { code: 'read', label_key: 'authorization.actions.read' },
    { code: 'update', label_key: 'authorization.actions.update' },
  ],
}

const emptyValues = { name: '', resource: '', action: '', description: null }

function mountForm(overrides: Record<string, unknown> = {}) {
  return mount(PermissionsForm, {
    props: {
      mode: 'create',
      catalog,
      initialValues: { ...emptyValues },
      existingCodes: [],
      ...overrides,
    },
  })
}

describe('PermissionsForm', () => {
  it('blocks an empty submit and marks the name invalid', async () => {
    const wrapper = mountForm()

    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('submit')).toBeUndefined()
    const name = wrapper.find('#permissionName')
    expect(name.attributes('aria-required')).toBe('true')
    expect(name.attributes('aria-invalid')).toBe('true')
  })

  it('previews the composed code from the selected resource and action', async () => {
    const wrapper = mountForm()

    await wrapper.find('#permissionResource').setValue('upload')
    await wrapper.find('#permissionAction').setValue('read')

    expect(wrapper.find('[data-testid="code-preview"]').text()).toBe('upload:read')
  })

  it('warns about a duplicate code before the submit', async () => {
    const wrapper = mountForm({ existingCodes: ['upload:read'] })

    await wrapper.find('#permissionResource').setValue('upload')
    await wrapper.find('#permissionAction').setValue('read')

    expect(wrapper.find('[data-testid="duplicate-warning"]').exists()).toBe(true)
  })

  it('submits the validated values', async () => {
    const wrapper = mountForm()

    await wrapper.find('#permissionName').setValue('Read uploads')
    await wrapper.find('#permissionResource').setValue('upload')
    await wrapper.find('#permissionAction').setValue('read')
    await wrapper.find('form').trigger('submit')

    expect(wrapper.emitted('submit')).toEqual([
      [
        {
          name: 'Read uploads',
          resource: 'upload',
          action: 'read',
          description: null,
        },
      ],
    ])
  })

  it('disables both selects in edit mode and keeps the code fixed', async () => {
    const wrapper = mountForm({
      mode: 'edit',
      initialValues: { name: 'Read uploads', resource: 'upload', action: 'read', description: null },
    })

    expect(wrapper.find('#permissionResource').attributes('disabled')).toBeDefined()
    expect(wrapper.find('#permissionAction').attributes('disabled')).toBeDefined()
    expect(wrapper.find('[data-testid="code-preview"]').text()).toBe('upload:read')

    // The fixed fields carry no error of their own.
    await wrapper.find('form').trigger('submit')
    expect(wrapper.find('[data-testid="resource-error"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="action-error"]').exists()).toBe(false)
  })
})
