<template>
  <div class="flex flex-col">
    <AppDataTableToolbar
      v-if="showToolbar"
      v-model="searchModel"
      :search-placeholder="searchPlaceholder"
      :total-label="totalLabel"
      :total="totalLabelCount"
      :show-total="showTotal"
      :enable-global-filter="enableGlobalFilter && mode === 'client'"
      :show-refresh="showRefresh"
      :refresh-label="refreshLabel"
      :loading="loading"
      @refresh="$emit('refresh')"
    >
      <template #left>
        <slot name="toolbar-left" />
      </template>
      <template #right>
        <slot name="toolbar-right" />
      </template>
    </AppDataTableToolbar>

    <div v-if="error" class="mb-4 rounded-box border border-error/20 bg-error/10 px-4 py-3 text-sm text-error">
      {{ error }}
    </div>

    <div class="overflow-auto rounded-box border border-base-content/10 bg-base-100" :style="containerStyle">
      <table class="table table-zebra table-pin-rows">
        <thead>
          <tr>
            <th
              v-for="col in table.getFlatHeaders()"
              :key="col.id"
              :class="headerClass(col.column.columnDef.meta)"
              @click="onHeaderClick(col)"
            >
              <span
                class="inline-flex items-center gap-1"
                :class="headerAlignClass(col.column.columnDef.meta)"
                :style="enableSorting && col.column.getCanSort() ? 'cursor: pointer; user-select: none;' : ''"
              >
                <FlexRender :render="col.column.columnDef.header" :props="col.getContext()" />
                <template v-if="enableSorting && col.column.getCanSort()">
                  <span
                    v-if="col.column.getIsSorted() === 'asc'"
                    class="icon-[tabler--chevron-up] size-3.5 text-primary"
                  ></span>
                  <span
                    v-else-if="col.column.getIsSorted() === 'desc'"
                    class="icon-[tabler--chevron-down] size-3.5 text-primary"
                  ></span>
                  <span v-else class="icon-[tabler--arrows-vertical] size-3.5 text-base-content/30"></span>
                </template>
              </span>
            </th>
          </tr>
        </thead>

        <tbody>
          <template v-if="loading || error || !rows.length">
            <tr>
              <td :colspan="table.getAllLeafColumns().length">
                <AppDataTableStatus
                  :state="statusState"
                  :label="statusLabel"
                />
              </td>
            </tr>
          </template>

          <template v-else>
            <tr
              v-for="row in rows"
              :key="row.id"
              class="cursor-pointer"
              @click="onRowClick(row)"
            >
              <td
                v-for="cell in row.getVisibleCells()"
                :key="cell.id"
                :class="cellClass(cell.column.columnDef.meta)"
              >
                <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <div
      v-if="showPagination && !loading && !error && rows.length"
      class="mt-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
    >
      <div class="flex items-center gap-2 text-sm text-base-content/60">
        <span>{{ totalLabelCount }} registros</span>
        <select
          :value="table.getState().pagination.pageSize"
          class="select select-sm select-bordered bg-base-100"
          @change="onPageSizeChange"
        >
          <option
            v-for="size in pageSizes"
            :key="size"
            :value="size"
          >
            {{ size }} / página
          </option>
        </select>
      </div>

      <nav class="flex items-center gap-x-1">
        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!table.getCanPreviousPage()"
          @click="table.setPageIndex(0)"
        >
          <span class="icon-[tabler--chevrons-left] size-4"></span>
        </button>
        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!table.getCanPreviousPage()"
          @click="table.previousPage()"
        >
          <span class="icon-[tabler--chevron-left] size-4 rtl:rotate-180"></span>
        </button>

        <template v-for="page in visiblePages" :key="page">
          <span
            v-if="page === '...'"
            class="px-2 text-base-content/40"
          >
            <span class="icon-[tabler--dots] size-4"></span>
          </span>
          <button
            v-else
            type="button"
            class="btn btn-sm"
            :class="page === table.getState().pagination.pageIndex + 1
              ? 'btn-primary'
              : 'btn-soft btn-square'"
            @click="table.setPageIndex(page - 1)"
          >
            {{ page }}
          </button>
        </template>

        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!table.getCanNextPage()"
          @click="table.nextPage()"
        >
          <span class="icon-[tabler--chevron-right] size-5 rtl:rotate-180"></span>
        </button>
        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!table.getCanNextPage()"
          @click="table.setPageIndex(table.getPageCount() - 1)"
        >
          <span class="icon-[tabler--chevrons-right] size-4"></span>
        </button>
      </nav>
    </div>

    <div v-if="$slots.footer" class="mt-3">
      <slot name="footer" />
    </div>
  </div>
</template>

<script setup lang="ts" generic="TData extends Record<string, any> = Record<string, any>">
import { computed, ref, watch } from 'vue'
import {
  FlexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useVueTable,
  type Row,
  type ColumnDef,
  type SortingState,
  type Header,
} from '@tanstack/vue-table'
import type {
  DataTableColumn,
  DataTableColumnMeta,
  DataTableRowAction,
} from '~/types/data-table'

interface Props {
  data: TData[]
  columns: DataTableColumn<TData>[]
  rowIdKey?: string
  searchPlaceholder?: string
  totalLabel?: string
  total?: number
  loading?: boolean
  error?: string
  emptyLabel?: string
  loadingLabel?: string
  errorLabel?: string
  rowActions?: DataTableRowAction<TData>[]
  height?: number | string
  enableSorting?: boolean
  enableGlobalFilter?: boolean
  mode?: 'client' | 'server'
  showToolbar?: boolean
  showTotal?: boolean
  showRefresh?: boolean
  refreshLabel?: string
  showPagination?: boolean
  pageSize?: number
  pageSizes?: number[]
}

const props = withDefaults(defineProps<Props>(), {
  rowIdKey: 'id',
  height: 480,
  enableSorting: true,
  enableGlobalFilter: true,
  mode: 'client',
  showToolbar: true,
  showTotal: true,
  showRefresh: false,
  refreshLabel: 'Atualizar',
  showPagination: true,
  pageSize: 10,
  pageSizes: () => [10, 20, 30, 50],
})

const emit = defineEmits<{
  refresh: []
  rowClick: [row: TData]
  'action:click': [action: DataTableRowAction<TData>, row: TData]
  'update:search': [value: string]
  'update:sorting': [state: SortingState]
  'update:pagination': [pageIndex: number, pageSize: number]
}>()

const searchModel = ref('')
const sorting = ref<SortingState>([])

watch(searchModel, (value) => emit('update:search', value))
watch(sorting, (value) => emit('update:sorting', value))

const table = useVueTable<TData>({
  get data() {
    return props.data
  },
  get columns() {
    return props.columns as unknown as ColumnDef<TData, any>[]
  },
  state: {
    get sorting() {
      return sorting.value
    },
    get globalFilter() {
      return props.mode === 'server' ? '' : searchModel.value
    },
  },
  onSortingChange: (updater) => {
    sorting.value = typeof updater === 'function' ? updater(sorting.value) : updater
  },
  onGlobalFilterChange: (updater) => {
    const next = typeof updater === 'function' ? updater(searchModel.value) : updater
    searchModel.value = next as string
  },
  enableSorting: props.enableSorting,
  enableGlobalFilter: props.mode === 'server' ? false : props.enableGlobalFilter,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getPaginationRowModel: getPaginationRowModel(),
  getRowId: (row, index) => String(row?.[props.rowIdKey] ?? index),
  initialState: {
    pagination: {
      pageIndex: 0,
      pageSize: props.pageSize,
    },
  },
})

const rows = computed(() => table.getRowModel().rows)

const containerStyle = computed(() => {
  const height = typeof props.height === 'number' ? `${props.height}px` : props.height
  return { maxHeight: height }
})

const totalLabelCount = computed(() => props.total ?? table.getPrePaginationRowModel().rows.length)

const statusState = computed<'loading' | 'error' | 'empty'>(() => {
  if (props.loading) return 'loading'
  if (props.error) return 'error'
  return 'empty'
})

const statusLabel = computed(() => {
  if (props.loading) return props.loadingLabel ?? 'Carregando registros...'
  if (props.error) return props.errorLabel ?? props.error ?? 'Erro ao carregar dados'
  return props.emptyLabel ?? 'Nenhum registro encontrado.'
})

const visiblePages = computed(() => {
  const total = table.getPageCount()
  const current = table.getState().pagination.pageIndex + 1
  if (total <= 7) return Array.from({ length: total }, (_, i) => i + 1)

  const pages: (number | string)[] = []
  pages.push(1)

  if (current > 3) pages.push('...')

  const start = Math.max(2, current - 1)
  const end = Math.min(total - 1, current + 1)
  for (let i = start; i <= end; i++) pages.push(i)

  if (current < total - 2) pages.push('...')
  pages.push(total)

  return pages
})

function headerClass(meta?: DataTableColumnMeta) {
  return [meta?.headerClass, 'text-sm font-semibold']
}

function headerAlignClass(meta?: DataTableColumnMeta) {
  switch (meta?.align) {
    case 'right':
      return 'justify-end'
    case 'center':
      return 'justify-center'
    default:
      return 'justify-start'
  }
}

function cellClass(meta?: DataTableColumnMeta) {
  return [meta?.cellClass, alignClass(meta?.align), meta?.truncate ? 'max-w-64 truncate' : '']
}

function alignClass(align?: 'left' | 'center' | 'right') {
  switch (align) {
    case 'right':
      return 'text-right'
    case 'center':
      return 'text-center'
    default:
      return 'text-left'
  }
}

function onHeaderClick(header: Header<TData, unknown>) {
  if (!props.enableSorting) return
  if (!header.column.getCanSort()) return
  header.column.getToggleSortingHandler()?.(undefined)
}

function onRowClick(row: Row<TData> | undefined) {
  if (!row) return
  emit('rowClick', row.original)
}

function onPageSizeChange(e: Event) {
  const value = Number((e.target as HTMLSelectElement).value)
  table.setPageSize(value)
}

defineExpose({
  table,
  reset: () => {
    searchModel.value = ''
    sorting.value = []
    table.setPageIndex(0)
  },
})
</script>
