// The RBAC types shared by the admin screens.
//
// The names mirror the backend models of the same concepts, so a screen reads
// the API payload without an intermediate translation layer.

/** A role of the canonical catalog, scoped or class level. */
export interface Role {
  id: string
  name: string
  resource_type: string | null
  resource_id: string | null
  created_at: string
  updated_at: string
}

/** The effect of a permission link: 0 allows, 1 denies. */
export type PermissionEffect = 0 | 1

/** A permission with the roles that carry it. */
export interface Permission {
  id: string
  code: string
  name: string
  description: string | null
  resource: string
  action: string
  roles: { role_id: string; role_name: string; effect: PermissionEffect }[]
}

/** The catalog of resources and actions the permission form offers. */
export interface PermissionCatalog {
  resources: { code: string; label_key: string }[]
  actions: { code: string; label_key: string }[]
}

/** A member of a role, with the email the backend decrypted. */
export interface RoleMember {
  id: string
  email: string
  display_name: string | null
}

/** One entry of the desired permission set of a role. */
export interface PermissionAssignment {
  code: string
  effect: PermissionEffect
}
