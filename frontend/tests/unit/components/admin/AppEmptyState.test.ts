import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import AppEmptyState from '@/app/components/admin/AppEmptyState.vue'

describe('AppEmptyState', () => {
  it('renders the title and the description from the props', () => {
    const wrapper = mount(AppEmptyState, {
      props: {
        title: 'Nenhuma permissão encontrada',
        description: 'Ajuste a busca ou crie uma nova permissão.',
      },
    })

    expect(wrapper.text()).toContain('Nenhuma permissão encontrada')
    expect(wrapper.text()).toContain('Ajuste a busca ou crie uma nova permissão.')
  })

  it('renders the description only when it is given', () => {
    const wrapper = mount(AppEmptyState, {
      props: { title: 'Vazio' },
    })

    expect(wrapper.text()).toContain('Vazio')
    expect(wrapper.find('[data-testid="empty-description"]').exists()).toBe(false)
  })

  it('renders the optional action slot', () => {
    const wrapper = mount(AppEmptyState, {
      props: { title: 'Vazio' },
      slots: {
        action: '<button type="button">Criar</button>',
      },
    })

    expect(wrapper.find('button').text()).toBe('Criar')
  })
})
