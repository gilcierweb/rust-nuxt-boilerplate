<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { Role } from '~/types/rbac'

const { t } = useI18n()

const props = defineProps<{
  /** The roles the selector offers, from the roles list. */
  roles: Role[]
  /** The ids the user holds, read back from the backend. */
  selectedIds: string[]
  /** The message of a refused save, surfaced without a refetch. */
  errorMessage?: string
  saving?: boolean
}>()

const emit = defineEmits<{
  save: [roleIds: string[]]
}>()

const checked = ref<string[]>([])

watch(
  () => props.selectedIds,
  (ids) => {
    checked.value = [...ids]
  },
  { immediate: true, deep: true },
)

const dirty = computed(
  () =>
    checked.value.length !== props.selectedIds.length ||
    checked.value.some((id) => !props.selectedIds.includes(id)),
)

function toggle(roleId: string, value: boolean | string) {
  const isChecked = value === true || value === 'true'
  if (isChecked) {
    if (!checked.value.includes(roleId)) checked.value.push(roleId)
    return
  }
  checked.value = checked.value.filter((id) => id !== roleId)
}
</script>

<template>
  <form class="space-y-4" @submit.prevent="emit('save', [...checked])">
    <div
      v-if="errorMessage"
      data-testid="role-selector-error"
      role="alert"
      class="rounded-box border border-error/40 bg-error/10 px-4 py-3 text-sm text-error"
    >
      {{ errorMessage }}
    </div>

    <div class="grid gap-2 sm:grid-cols-2">
      <label
        v-for="role in roles"
        :key="role.id"
        class="flex cursor-pointer items-center gap-3 rounded-box border border-base-content/10 px-3 py-2 hover:bg-base-200/40"
        :class="{ 'border-primary/40 bg-primary/5': checked.includes(role.id) }"
      >
        <input
          type="checkbox"
          class="checkbox checkbox-sm"
          :value="role.id"
          :checked="checked.includes(role.id)"
          @change="toggle(role.id, ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-base-content">{{ role.name }}</span>
        <span v-if="role.resource_type" class="badge badge-ghost badge-sm">
          {{ role.resource_type }}
        </span>
      </label>
    </div>

    <div class="flex items-center justify-end gap-3 border-t border-base-content/10 pt-4">
      <span v-if="dirty" class="mr-auto text-xs text-base-content/60">
        {{ t('admin.users.roles.unsaved') }}
      </span>
      <button type="submit" class="btn btn-primary" :disabled="saving">
        <span v-if="saving" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
        <span v-else class="icon-[tabler--check] size-5"></span>
        {{ saving ? t('admin.users.roles.saving') : t('admin.users.roles.save') }}
      </button>
    </div>
  </form>
</template>
