import type { Component } from 'vue'
import type { ColumnDef, ColumnDefBase } from '@tanstack/vue-table'
import { normalizeResourceResponse } from '~/utils/admin-resources'

export type DataTableFilterMode = 'client' | 'server'

export type DataTableCellFormat =
  | 'text'
  | 'datetime'
  | 'date'
  | 'boolean'
  | 'json'
  | 'numeric'
  | 'currency'

export type DataTableColumnAlign = 'left' | 'center' | 'right'

export interface DataTableRowAction<TData = Record<string, any>> {
  key: string
  label: string
  icon?: string
  to?: (row: TData) => string
  href?: (row: TData) => string
  danger?: boolean
  disabled?: (row: TData) => boolean
  pending?: (row: TData) => boolean
  onClick?: (row: TData) => void | Promise<void>
}

export interface DataTableColumnMeta {
  align?: DataTableColumnAlign
  format?: DataTableCellFormat
  filterType?: 'text' | 'select'
  selectOptions?: { label: string; value: string | number }[]
  truncate?: boolean
  headerClass?: string
  cellClass?: string
}

export type DataTableColumn<TData extends Record<string, any>> = ColumnDef<TData, any> & {
  meta?: DataTableColumnMeta
}

export interface DataTableRowSelection {
  rowIdKey: string
}

export interface DataTableState {
  page: number
  pageSize: number
  search: string
  sortBy: string | null
  sortDir: 'asc' | 'desc' | null
}

export interface DataTableProps<TData extends Record<string, any>> {
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
  rowSelection?: DataTableRowSelection
  estimateRowHeight?: number
  overscan?: number
  height?: number | string
  enableSorting?: boolean
  enableGlobalFilter?: boolean
  mode?: DataTableFilterMode
  toolbarClass?: string
  showToolbar?: boolean
}

export function buildDataTableColumns<TData extends Record<string, any>>(
  source: DataTableColumn<TData>[],
): DataTableColumn<TData>[] {
  return source
}

export function columnHelper<T extends Record<string, any>>() {
  return {
    accessor<K extends keyof T & string>(key: K, config: ColumnDefBase<T, T[K]> & { meta?: DataTableColumnMeta } = {}): DataTableColumn<T> {
      return {
        id: key,
        accessorKey: key,
        ...config,
      } as DataTableColumn<T>
    },
    display(id: string, config: ColumnDefBase<T, unknown> & { meta?: DataTableColumnMeta } = {}): DataTableColumn<T> {
      return {
        id,
        ...config,
      } as DataTableColumn<T>
    },
  }
}

export function normalizeTableData<T extends Record<string, any> = Record<string, any>>(payload: any): T[] {
  return normalizeResourceResponse(payload) as T[]
}

export function defaultRowIdKey(row: Record<string, any>, fallback = 'id'): string {
  return String(row?.[fallback] ?? row?.uuid ?? row?.key ?? Math.random().toString(36).slice(2))
}

export function renderActionLink(action: DataTableRowAction, row: Record<string, any>): { to?: string; href?: string } {
  if (action.to) return { to: action.to(row) }
  if (action.href) return { href: action.href(row) }
  return {}
}

export type { ColumnDef, Component }
