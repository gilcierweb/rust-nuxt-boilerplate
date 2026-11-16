import { describe, expect, it, vi } from 'vitest'
import { h, mount } from '@vue/test-utils'

// The shared setup replaces `vue` with non reactive shims for the utility
// tests. A selection needs the real reactivity of the TanStack row state.
vi.unmock('vue')

import AppDataTable from '@/app/components/AppDataTable.vue'
import AppBulkActions from '@/app/components/AppBulkActions.vue'
import AppDataTableToolbar from '@/app/components/AppDataTableToolbar.vue'
import AppDataTableStatus from '@/app/components/AppDataTableStatus.vue'
import type { DataTableColumn } from '~/types/data-table'

type Row = { id: string; name: string }

const data: Row[] = [
  { id: '1', name: 'Alpha' },
  { id: '2', name: 'Beta' },
]

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
      pageCount: 1,
      total: 2,
      ...props,
    },
    global: {
      components: {
        AppBulkActions,
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

function lastEmission(wrapper: ReturnType<typeof mountTable>, event: string) {
  const emitted = wrapper.emitted(event) ?? []
  return emitted[emitted.length - 1]?.[0]
}

describe('AppDataTable row selection', () => {
  it('renders no checkbox column by default', () => {
    const wrapper = mountTable()

    expect(wrapper.find('[data-testid="select-all"]').exists()).toBe(false)
    expect(wrapper.find('[data-testid="select-row"]').exists()).toBe(false)
  })

  it('toggles one row without opening it', async () => {
    const wrapper = mountTable({ enableSelection: true })

    const rowCheckbox = wrapper.findAll('[data-testid="select-row"]')[0]
    await rowCheckbox.trigger('change')

    expect(lastEmission(wrapper, 'update:rowSelection')).toEqual({ '1': true })
    expect(wrapper.emitted('rowClick')).toBeFalsy()
  })

  it('selects every row of the page from the header checkbox', async () => {
    const wrapper = mountTable({ enableSelection: true })

    await wrapper.find('[data-testid="select-all"]').trigger('change')

    expect(lastEmission(wrapper, 'update:rowSelection')).toEqual({ '1': true, '2': true })

    // The next click clears the page again: the state travels through the
    // v-model, so the bar disappears once the selection empties.
    await wrapper.find('[data-testid="select-all"]').trigger('change')

    expect(lastEmission(wrapper, 'update:rowSelection')).toEqual({})
  })

  it('marks the header checkbox indeterminate on a partial selection', async () => {
    const wrapper = mountTable({ enableSelection: true })

    await wrapper.findAll('[data-testid="select-row"]')[0].trigger('change')

    const selectAll = wrapper.find('[data-testid="select-all"]').element as HTMLInputElement
    expect(selectAll.indeterminate).toBe(true)
  })

  it('shows the bulk action bar with the count and clears the selection', async () => {
    const wrapper = mountTable({ enableSelection: true })

    expect(wrapper.find('[data-testid="bulk-actions"]').exists()).toBe(false)

    await wrapper.findAll('[data-testid="select-row"]')[0].trigger('change')

    const bar = wrapper.find('[data-testid="bulk-actions"]')
    expect(bar.exists()).toBe(true)
    expect(bar.text()).toContain('1')

    await wrapper.find('[data-testid="bulk-clear"]').trigger('click')

    expect(lastEmission(wrapper, 'update:rowSelection')).toEqual({})
  })

  it('keeps the page v-model as the source of the selection', async () => {
    const wrapper = mountTable({ enableSelection: true, rowSelection: { '2': true } })

    const rowCheckboxes = wrapper.findAll('[data-testid="select-row"]')
    expect((rowCheckboxes[1].element as HTMLInputElement).checked).toBe(true)
  })
})