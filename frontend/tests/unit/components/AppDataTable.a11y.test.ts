import { describe, expect, it, vi } from 'vitest'
import { h, mount } from '@vue/test-utils'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A mounted table needs the real reactivity of its sorting state.
vi.unmock('vue')

import AppDataTable from '@/app/components/AppDataTable.vue'
import AppDataTableToolbar from '@/app/components/AppDataTableToolbar.vue'
import AppDataTableStatus from '@/app/components/AppDataTableStatus.vue'
import type { DataTableColumn } from '~/types/data-table'

type Row = { id: string; name: string }

const data: Row[] = [{ id: '1', name: 'Alpha' }]

const columns: DataTableColumn<Row>[] = [
  {
    id: 'name',
    accessorKey: 'name',
    header: () => 'Name',
    cell: (info: any) => info.getValue(),
  },
]

function mountTable(props: Record<string, unknown> = {}) {
  return mount(AppDataTable as any, {
    props: {
      data,
      columns: columns as any,
      mode: 'server',
      page: 1,
      pageSize: 10,
      pageCount: 3,
      total: 30,
      ...props,
    },
    global: {
      components: {
        AppDataTableToolbar,
        AppDataTableStatus,
      },
      // The identity translation keeps the assertions on the i18n keys.
      mocks: {
        $t: (key: string) => key,
      },
    },
  })
}

describe('AppDataTable accessibility', () => {
  it('renders the sortable header as a button that reports its state', async () => {
    const wrapper = mountTable({ enableSorting: true })

    const header = wrapper.find('thead button')
    expect(header.exists()).toBe(true)
    expect(header.attributes('aria-sort')).toBe('none')

    await header.trigger('click')

    expect(wrapper.emitted('update:sorting')).toBeTruthy()
  })

  it('keeps the search filter visible in server mode', () => {
    const wrapper = mountTable({ enableGlobalFilter: true })

    expect(wrapper.find('input[type="search"]').exists()).toBe(true)
  })

  it('gives every pagination button an accessible name', () => {
    const wrapper = mountTable({ showPagination: true })

    const buttons = wrapper.findAll('nav button')
    expect(buttons.length).toBeGreaterThan(0)
    for (const button of buttons) {
      const name = button.attributes('aria-label') ?? button.text().trim()
      expect(name, 'a pagination button without an accessible name').not.toBe('')
    }
  })

  it('binds the page size select to its label', () => {
    const wrapper = mountTable()

    const select = wrapper.find('select')
    const id = select.attributes('id')
    expect(id).toBeTruthy()
    expect(wrapper.find(`label[for="${id}"]`).exists()).toBe(true)
  })
})
