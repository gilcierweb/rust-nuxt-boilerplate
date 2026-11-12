<script setup lang="ts">
import type { Permission } from '~/types/rbac'

type MatrixEffect = 'allow' | 'deny' | 'none'

/** One column of the matrix: the role id keys the column and the name labels it. */
export interface MatrixRole {
  id: string
  name: string
}

const { t } = useI18n()

defineProps<{
  /** The roles of the columns, in catalog order. */
  roles: MatrixRole[]
  /** The permissions of the rows, with their links. */
  permissions: Permission[]
}>()

/**
 * The effect a role holds over a permission: the link when it exists, and
 * `none` when the role does not carry the permission at all. The two effects
 * are different states, so a missing link never reads as a denial.
 *
 * The lookup keys on the role id, never on the name: the unique index of the
 * catalog is (name, scope), so two scoped roles may share a name and only the
 * id tells the columns apart.
 */
function effectOf(permission: Permission, role: MatrixRole): MatrixEffect {
  const link = permission.roles.find((entry) => entry.role_id === role.id)
  if (!link) return 'none'
  return link.effect === 0 ? 'allow' : 'deny'
}

function effectKey(effect: MatrixEffect): string {
  if (effect === 'allow') return t('admin.permissions.effect.allow')
  if (effect === 'deny') return t('admin.permissions.effect.deny')
  return t('admin.permissions.effect.none')
}
</script>

<template>
  <div class="overflow-x-auto">
    <table class="table table-sm" data-testid="permission-matrix">
      <thead>
        <tr>
          <th scope="col">{{ t('admin.permissions.table.code') }}</th>
          <th v-for="role in roles" :key="role.id" scope="col" class="text-center">
            {{ role.name }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="permission in permissions" :key="permission.code">
          <th scope="row" class="font-mono text-xs">
            {{ permission.code }}
          </th>
          <td v-for="role in roles" :key="role.id" class="text-center">
            <span
              class="badge badge-sm"
              :data-testid="`cell-${permission.code}-${role.id}`"
              :data-effect="effectOf(permission, role)"
              :class="{
                'badge-success': effectOf(permission, role) === 'allow',
                'badge-error': effectOf(permission, role) === 'deny',
                'badge-ghost': effectOf(permission, role) === 'none',
              }"
            >
              {{ effectKey(effectOf(permission, role)) }}
            </span>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
