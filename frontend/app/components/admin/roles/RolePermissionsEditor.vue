<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { splitCode } from '~/utils/rbac'
import type { Permission, PermissionAssignment, PermissionEffect } from '~/types/rbac'

type Option = {
  value: 'none' | 'allow' | 'deny'
  effect: PermissionEffect | undefined
  labelKey: string
}

const { t } = useI18n()

const props = defineProps<{
  /** The permission catalog, already carrying every role link. */
  permissions: Permission[]
  /** The links this role holds, read back from the backend. */
  assignments: PermissionAssignment[]
  saving?: boolean
}>()

const emit = defineEmits<{
  save: [assignments: PermissionAssignment[]]
}>()

/**
 * The codes a subject must hold to repair the system, mirrored from the
 * lockout guard of the backend. An edit that would leave no role holding both
 * is refused there, so the editor warns before the request leaves.
 */
const ADMINISTRATOR_CODES = ['roles:read', 'roles:update']

const options: Option[] = [
  { value: 'none', effect: undefined, labelKey: 'admin.permissions.effect.none' },
  { value: 'allow', effect: 0, labelKey: 'admin.permissions.effect.allow' },
  { value: 'deny', effect: 1, labelKey: 'admin.permissions.effect.deny' },
]

/** The chosen effect per code: an absent code reads as none. */
const selected = ref<Record<string, PermissionEffect>>({})

watch(
  () => props.assignments,
  (links) => {
    selected.value = Object.fromEntries(links.map((link) => [link.code, link.effect]))
  },
  { immediate: true, deep: true },
)

const groups = computed(() => {
  const byResource = new Map<string, Permission[]>()
  for (const permission of props.permissions) {
    const resource = splitCode(permission.code)?.resource ?? permission.code
    const bucket = byResource.get(resource)
    if (bucket) bucket.push(permission)
    else byResource.set(resource, [permission])
  }
  return [...byResource.entries()].map(([resource, items]) => ({ resource, permissions: items }))
})

const loadedCodes = computed(() => props.assignments.map((link) => link.code))

/** Whether the chosen set differs from the one the role holds. */
const dirty = computed(() => {
  const chosen = Object.entries(selected.value).map(([code, effect]) => `${code}:${effect}`)
  const loaded = props.assignments.map((link) => `${link.code}:${link.effect}`)
  return (
    chosen.length !== loaded.length || chosen.some((entry) => !loaded.includes(entry))
  )
})

const wouldLockOut = computed(
  () =>
    ADMINISTRATOR_CODES.every((code) => loadedCodes.value.includes(code)) &&
    // A deny link takes the authority away as surely as a removed one: the
    // engine lets deny beat allow, so only the allow effect keeps the code.
    !ADMINISTRATOR_CODES.every((code) => selected.value[code] === 0),
)

/**
 * The catalog is asked one page at a time and the backend clamps the page at
 * one hundred rows. A save replaces every link of the role, so an incomplete
 * set would silently drop the links the admin never saw: the editor says so
 * instead of hiding it.
 */
const catalogTruncated = computed(() => props.permissions.length >= 100)

function valueOf(code: string): Option['value'] {
  const effect = selected.value[code]
  if (effect === 0) return 'allow'
  if (effect === 1) return 'deny'
  return 'none'
}

function choose(code: string, effect: PermissionEffect | undefined) {
  if (effect === undefined) delete selected.value[code]
  else selected.value[code] = effect
}

function save() {
  // The PUT replaces every link, so the payload carries the whole desired set
  // in catalog order and omits the codes left at none.
  const assignments: PermissionAssignment[] = []
  for (const group of groups.value) {
    for (const permission of group.permissions) {
      const effect = selected.value[permission.code]
      if (effect !== undefined) assignments.push({ code: permission.code, effect })
    }
  }
  emit('save', assignments)
}
</script>

<template>
  <form class="space-y-6" @submit.prevent="save">
    <div
      v-if="catalogTruncated"
      data-testid="catalog-truncated-warning"
      role="alert"
      class="alert alert-warning"
    >
      <span class="icon-[tabler--alert-triangle] size-5"></span>
      <span>{{ t('admin.roles.permissions.catalog_truncated') }}</span>
    </div>

    <div
      v-for="group in groups"
      :key="group.resource"
      :data-testid="'resource-group'"
      class="rounded-box border border-base-content/10"
    >
      <p class="border-b border-base-content/10 px-4 py-2 text-xs font-semibold uppercase tracking-[0.18em] text-base-content/60">
        {{ group.resource }}
      </p>
      <div
        v-for="permission in group.permissions"
        :key="permission.code"
        :data-testid="`cell-${permission.code}`"
        :data-value="valueOf(permission.code)"
        class="flex flex-col gap-2 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
      >
        <div>
          <p class="text-sm font-medium text-base-content">{{ permission.name }}</p>
          <p class="font-mono text-xs text-base-content/50">{{ permission.code }}</p>
        </div>
        <div role="group" class="join">
          <button
            v-for="option in options"
            :key="option.value"
            type="button"
            class="join-item btn btn-sm"
            :class="{ 'btn-active btn-primary': valueOf(permission.code) === option.value }"
            :data-testid="`option-${option.value}`"
            :aria-pressed="valueOf(permission.code) === option.value ? 'true' : 'false'"
            @click="choose(permission.code, option.effect)"
          >
            {{ t(option.labelKey) }}
          </button>
        </div>
      </div>
    </div>

    <div
      v-if="wouldLockOut"
      data-testid="lockout-warning"
      role="alert"
      class="rounded-box border border-warning/40 bg-warning/10 px-4 py-3 text-sm"
    >
      <p class="font-semibold">{{ t('admin.roles.permissions.locked_title') }}</p>
      <p>{{ t('admin.roles.permissions.locked_warning') }}</p>
    </div>

    <div class="flex items-center justify-end gap-3 border-t border-base-content/10 pt-4">
      <span v-if="dirty" class="mr-auto text-xs text-base-content/60">
        {{ t('admin.roles.permissions.unsaved') }}
      </span>
      <button type="submit" class="btn btn-primary" :disabled="saving">
        <span v-if="saving" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
        <span v-else class="icon-[tabler--check] size-5"></span>
        {{ saving ? t('admin.roles.permissions.saving') : t('admin.roles.permissions.save') }}
      </button>
    </div>
  </form>
</template>
