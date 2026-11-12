import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import ConfirmDialog from '@/app/components/admin/ConfirmDialog.vue'

describe('ConfirmDialog', () => {
  const props = {
    open: true,
    title: 'Excluir cargo',
    message: 'admin',
    confirmLabel: 'Excluir',
    cancelLabel: 'Cancelar',
  }

  it('names the target and the consequence', () => {
    const wrapper = mount(ConfirmDialog, { props })

    expect(wrapper.text()).toContain('Excluir cargo')
    expect(wrapper.text()).toContain('admin')
  })

  it('emits confirm from the accept button', async () => {
    const wrapper = mount(ConfirmDialog, { props })

    await wrapper.find('[data-testid="confirm-accept"]').trigger('click')

    expect(wrapper.emitted('confirm')).toHaveLength(1)
  })

  it('emits cancel from the dismiss button', async () => {
    const wrapper = mount(ConfirmDialog, { props })

    await wrapper.find('[data-testid="confirm-cancel"]').trigger('click')

    expect(wrapper.emitted('cancel')).toHaveLength(1)
  })

  it('renders nothing while it is closed', () => {
    const wrapper = mount(ConfirmDialog, { props: { ...props, open: false } })

    expect(wrapper.find('[data-testid="confirm-accept"]').exists()).toBe(false)
  })
})
