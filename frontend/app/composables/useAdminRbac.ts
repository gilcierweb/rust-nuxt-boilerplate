import type {
  Permission,
  PermissionAssignment,
  PermissionCatalog,
  Role,
  RoleMember,
} from '~/types/rbac'

import { useApi } from './useApi'

/** A role summary as the user payload carries it. */
export interface RoleSummary {
  id: string
  name: string
}

/** The page shape of every admin list endpoint. */
export interface Paginated<T> {
  data: T[]
  pagination: {
    page: number
    per_page: number
    total: number
    total_pages: number
    has_next: boolean
    has_prev: boolean
  }
}

/** The query string the admin lists accept: search and the allowlisted sort. */
export interface AdminListQuery {
  page?: number
  per_page?: number
  q?: string
  sort_by?: string
  sort_dir?: string
}

/** The write payload of a permission. */
export interface PermissionWritePayload {
  name: string
  resource: string
  action: string
  description?: string | null
}

function dropEmptyValues(query: AdminListQuery): Record<string, string | number> {
  const result: Record<string, string | number> = {}
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== null && value !== '') {
      result[key] = value
    }
  }
  return result
}

/**
 * The typed wrappers of the admin RBAC endpoints.
 *
 * The names follow the Rails actions of the backend handlers: `list` for a
 * collection, `create`, `update` and `remove` for the writes, and the member
 * reads keep the semantic name of the sub resource.
 */
export function useAdminRbac() {
  const api = useApi()

  const roles = {
    list: (query: AdminListQuery = {}) =>
      api.get<Paginated<Role>>('/admin/roles', { query: dropEmptyValues(query) }),
  }

  const permissions = {
    list: (query: AdminListQuery = {}) =>
      api.get<Paginated<Permission>>('/admin/permissions', {
        query: dropEmptyValues(query),
      }),
    catalog: () => api.get<PermissionCatalog>('/admin/permissions/catalog'),
    create: (payload: PermissionWritePayload) =>
      api.post<Permission>('/admin/permissions', { body: payload }),
    update: (id: string, payload: Partial<PermissionWritePayload>) =>
      api.patch<Permission>(`/admin/permissions/${id}`, { body: payload }),
    remove: (id: string) => api.delete<void>(`/admin/permissions/${id}`),
  }

  const rolePermissions = {
    get: async (roleId: string): Promise<PermissionAssignment[]> => {
      const response = await api.get<{ permissions: PermissionAssignment[] }>(
        `/admin/roles/${roleId}/permissions`,
      )
      return response.permissions
    },
    set: (roleId: string, assignments: PermissionAssignment[]) =>
      api.put<{ updated: number }>(`/admin/roles/${roleId}/permissions`, {
        body: { permissions: assignments },
      }),
  }

  const roleMembers = {
    list: (roleId: string) =>
      api.get<RoleMember[]>(`/admin/roles/${roleId}/members`),
  }

  const userRoles = {
    get: async (userId: string): Promise<RoleSummary[]> => {
      const response = await api.get<{ roles: RoleSummary[] }>(
        `/admin/users/${userId}/roles`,
      )
      return response.roles
    },
    set: (userId: string, roleIds: string[]) =>
      api.put<{ roles: RoleSummary[] }>(`/admin/users/${userId}/roles`, {
        body: { role_ids: roleIds },
      }),
  }

  return { roles, permissions, rolePermissions, roleMembers, userRoles }
}
