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
      v-if="showPagination"
      class="mt-3 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between"
    >
      <div class="flex items-center gap-2 text-sm text-base-content/60">
        <span v-if="totalLabelCount !== null">
          {{ totalLabelCount }} {{ totalLabelCount === 1 ? 'registro' : 'registros' }}
        </span>
        <select
          :value="currentPageSize"
          :disabled="loading"
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
          :disabled="!canPreviousPage || loading"
          @click="goToPage(1)"
        >
          <span class="icon-[tabler--chevrons-left] size-4"></span>
        </button>
        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!canPreviousPage || loading"
          @click="goToPage(currentPage - 1)"
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
            :class="page === currentPage
              ? 'btn-primary'
              : 'btn-soft btn-square'"
            :disabled="loading"
            @click="goToPage(page)"
          >
            {{ page }}
          </button>
        </template>

        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!canNextPage || loading"
          @click="goToPage(currentPage + 1)"
        >
          <span class="icon-[tabler--chevron-right] size-5 rtl:rotate-180"></span>
        </button>
        <button
          type="button"
          class="btn btn-soft btn-sm"
          :disabled="!canNextPage || loading"
          @click="goToPage(totalPages)"
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
  total?: number | null
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
  page?: number
  pageSize?: number
  pageSizes?: number[]
  pageCount?: number
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
  page: 1,
  pageSize: 10,
  pageSizes: () => [10, 20, 30, 50],
  pageCount: 0,
  total: null,
})

const emit = defineEmits<{
  refresh: []
  rowClick: [row: TData]
  'action:click': [action: DataTableRowAction<TData>, row: TData]
  'update:search': [value: string]
  'update:sorting': [state: SortingState]
  'update:page': [page: number]
  'update:pageSize': [size: number]
}>()

const searchModel = ref('')
const sorting = ref<SortingState>([])

watch(searchModel, (value) => emit('update:search', value))
watch(sorting, (value) => emit('update:sorting', value))

const isServer = computed(() => props.mode === 'server')

const clientTable = useVueTable<TData>({
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
      return isServer.value ? '' : searchModel.value
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
  enableGlobalFilter: isServer.value ? false : props.enableGlobalFilter,
  getCoreRowModel: getCoreRowModel(),
  getSortedRowModel: getSortedRowModel(),
  getFilteredRowModel: getFilteredRowModel(),
  getRowId: (row, index) => String(row?.[props.rowIdKey] ?? index),
})

const table = computed(() => {
  if (isServer.value) return clientTable
  return clientTable
})

const rows = computed(() => clientTable.getRowModel().rows)

const containerStyle = computed(() => {
  const height = typeof props.height === 'number' ? `${props.height}px` : props.height
  return { maxHeight: height }
})

const totalLabelCount = computed(() => {
  if (props.total !== null && props.total !== undefined) return props.total
  if (isServer.value) return 0
  return clientTable.getPrePaginationRowModel().rows.length
})

const currentPage = computed(() => {
  if (isServer.value) return Math.max(1, props.page)
  return clientTable.getState().pagination.pageIndex + 1
})

const currentPageSize = computed(() => {
  if (isServer.value) return props.pageSize
  return clientTable.getState().pagination.pageSize
})

const totalPages = computed(() => {
  if (isServer.value) return Math.max(1, props.pageCount || 1)
  return Math.max(1, clientTable.getPageCount())
})

const canPreviousPage = computed(() => currentPage.value > 1)
const canNextPage = computed(() => currentPage.value < totalPages.value)

const visiblePages = computed(() => {
  const total = totalPages.value
  const current = currentPage.value
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

function goToPage(page: number) {
  if (props.loading) return
  const target = Math.max(1, Math.min(totalPages.value, page))
  if (target === currentPage.value) return

  if (isServer.value) {
    emit('update:page', target)
  } else {
    clientTable.setPageIndex(target - 1)
  }
}

function onPageSizeChange(e: Event) {
  const value = Number((e.target as HTMLSelectElement).value)
  if (!value || value === currentPageSize.value) return

  if (isServer.value) {
    emit('update:pageSize', value)
    emit('update:page', 1)
  } else {
    clientTable.setPageSize(value)
  }
}

defineExpose({
  table: clientTable,
  reset: () => {
    searchModel.value = ''
    sorting.value = []
    if (isServer.value) {
      emit('update:page', 1)
    } else {
      clientTable.setPageIndex(0)
    }
  },
})
</script>
