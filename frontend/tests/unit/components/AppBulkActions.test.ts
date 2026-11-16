import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import AppBulkActions from '@/app/components/AppBulkActions.vue'

describe('AppBulkActions', () => {
  const props = { count: 3, label: 'admin.common.bulkActions.selectedCount' }

  function mountBar(overrides: Record<string, unknown> = {}) {
    return mount(AppBulkActions, {
      props: { ...props, ...overrides },
      global: {
        // The identity translation keeps the assertions on the i18n keys.
        mocks: {
          $t: (key: string) => key,
        },
      },
    })
  }

  it('shows the selected count beside the label', () => {
    const wrapper = mountBar()

    expect(wrapper.find('[data-testid="bulk-actions"]').exists()).toBe(true)
    expect(wrapper.text()).toContain('3')
    expect(wrapper.text()).toContain('admin.common.bulkActions.selectedCount')
  })

  it('renders the actions the page passes through the slot', () => {
    const wrapper = mount(AppBulkActions, {
      props,
      global: { mocks: { $t: (key: string) => key } },
      slots: {
        default: '<button data-testid="page-action">Excluir</button>',
      },
    })

    expect(wrapper.find('[data-testid="page-action"]').exists()).toBe(true)
  })

  it('emits clear from the clear button', async () => {
    const wrapper = mountBar()

    await wrapper.find('[data-testid="bulk-clear"]').trigger('click')

    expect(wrapper.emitted('clear')).toHaveLength(1)
  })

  it('emits clear on Escape', async () => {
    const wrapper = mountBar()

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('clear')).toHaveLength(1)
  })

  it('renders nothing without a selection', () => {
    const wrapper = mountBar({ count: 0 })

    expect(wrapper.find('[data-testid="bulk-actions"]').exists()).toBe(false)
  })
})