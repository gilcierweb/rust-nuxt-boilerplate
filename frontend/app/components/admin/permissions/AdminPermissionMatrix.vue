<script setup lang="ts">
import type { Permission } from '~/types/rbac'

type MatrixEffect = 'allow' | 'deny' | 'none'

const { t } = useI18n()

defineProps<{
  /** The role names of the columns, in catalog order. */
  roles: string[]
  /** The permissions of the rows, with their links. */
  permissions: Permission[]
}>()

/**
 * The effect a role holds over a permission: the link when it exists, and
 * `none` when the role does not carry the permission at all. The two effects
 * are different states, so a missing link never reads as a denial.
 */
function effectOf(permission: Permission, role: string): MatrixEffect {
  const link = permission.roles.find((entry) => entry.role_name === role)
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
          <th v-for="role in roles" :key="role" scope="col" class="text-center">
            {{ role }}
          </th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="permission in permissions" :key="permission.code">
          <th scope="row" class="font-mono text-xs">
            {{ permission.code }}
          </th>
          <td v-for="role in roles" :key="role" class="text-center">
            <span
              class="badge badge-sm"
              :data-testid="`cell-${permission.code}-${role}`"
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
