import type { Permission } from '~/types/rbac'

/** The permission row as the catalog table renders it. */
export interface PermissionTableRow {
  id: string
  name: string
  code: string
  description: string | null
  resource: string
  action: string
  roles: string[]
}

/**
 * Compiles an API permission into a table row.
 *
 * The screen shows one line per permission and lists the roles that carry it,
 * so the link array collapses into the role names in the order the API
 * answers them. Nothing else is compiled: the catalog list carries no
 * timestamp, so a column that reads one would render an empty cell.
 */
export function toPermissionTableRow(permission: Permission): PermissionTableRow {
  return {
    id: permission.id,
    name: permission.name,
    code: permission.code,
    description: permission.description,
    resource: permission.resource,
    action: permission.action,
    roles: permission.roles.map((link) => link.role_name),
  }
}

/** Compiles a page of permissions into table rows. */
export function toPermissionTableRows(permissions: Permission[]): PermissionTableRow[] {
  return permissions.map(toPermissionTableRow)
}
