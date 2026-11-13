<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { PermissionCatalog } from '~/types/rbac'

type FormValues = {
  name: string
  resource: string
  action: string
  description: string | null
}

const { t } = useI18n()
const localePath = useLocalePath()

const props = defineProps<{
  mode: 'create' | 'edit'
  /** The resources and actions the form offers, from the catalog endpoint. */
  catalog: PermissionCatalog
  initialValues: FormValues
  /**
   * The codes of the other permissions, so the form can warn about a
   * collision before the submit. The code of the row being edited is not
   * part of the list, otherwise the edit would warn about itself.
   */
  existingCodes: string[]
  saving?: boolean
}>()

const emit = defineEmits<{
  submit: [values: FormValues]
}>()

const name = ref(props.initialValues.name)
const resource = ref(props.initialValues.resource)
const action = ref(props.initialValues.action)
const description = ref(props.initialValues.description ?? '')

watch(
  () => props.initialValues,
  (values) => {
    name.value = values.name
    resource.value = values.resource
    action.value = values.action
    description.value = values.description ?? ''
  },
  { deep: true },
)

/** The composed code, live: both halves chosen, or an empty string. */
const code = computed(() =>
  resource.value && action.value ? `${resource.value}:${action.value}` : '',
)

const duplicate = computed(() => code.value !== '' && props.existingCodes.includes(code.value))

/**
 * The mistakes of the payload. The two selects are fixed in edit mode, so they
 * are never wrong there: their values come from the stored row.
 */
const errors = computed(() => {
  const next: Record<string, string> = {}
  const trimmed = name.value.trim()
  if (trimmed === '') next.name = t('admin.permissions.form.name_required')
  else if (trimmed.length > 255) next.name = t('admin.permissions.form.name_too_long')
  if (props.mode === 'create') {
    if (resource.value === '') next.resource = t('admin.permissions.form.resource_required')
    if (action.value === '') next.action = t('admin.permissions.form.action_required')
  }
  return next
})

function submit() {
  if (Object.keys(errors.value).length > 0) return
  const trimmedDescription = description.value.trim()
  emit('submit', {
    name: name.value.trim(),
    resource: resource.value,
    action: action.value,
    description: trimmedDescription === '' ? null : trimmedDescription,
  })
}
</script>

<template>
  <form class="needs-validation peer" novalidate @submit.prevent="submit">
    <div class="space-y-6">
      <div>
        <label class="label-text font-semibold" for="permissionName">
          {{ t('admin.permissions.form.name') }} <span class="text-error">*</span>
        </label>
        <input
          id="permissionName"
          v-model="name"
          type="text"
          maxlength="255"
          class="input w-full"
          :class="{ 'is-invalid': errors.name }"
          :placeholder="t('admin.permissions.form.name_placeholder')"
          :aria-required="true"
          :aria-invalid="errors.name ? 'true' : 'false'"
          :aria-describedby="errors.name ? 'permissionNameError' : 'permissionNameHint'"
        />
        <span
          v-if="errors.name"
          id="permissionNameError"
          role="alert"
          class="text-error text-xs mt-1 block"
        >
          {{ errors.name }}
        </span>
        <span v-else id="permissionNameHint" class="text-base-content/60 text-xs mt-1 block">
          {{ t('admin.permissions.form.name_hint') }}
        </span>
      </div>

      <div class="grid gap-6 md:grid-cols-2">
        <div>
          <label class="label-text font-semibold" for="permissionResource">
            {{ t('admin.permissions.form.resource') }} <span class="text-error">*</span>
          </label>
          <select
            id="permissionResource"
            v-model="resource"
            class="select w-full"
            :class="{ 'is-invalid': errors.resource }"
            :disabled="mode === 'edit'"
            :aria-required="true"
            :aria-invalid="errors.resource ? 'true' : 'false'"
            :aria-describedby="errors.resource ? 'permissionResourceError' : undefined"
          >
            <option value="">{{ t('admin.permissions.form.select_resource') }}</option>
            <option v-for="item in catalog.resources" :key="item.code" :value="item.code">
              {{ t(item.label_key) }}
            </option>
          </select>
          <span
            v-if="errors.resource"
            id="permissionResourceError"
            data-testid="resource-error"
            role="alert"
            class="text-error text-xs mt-1 block"
          >
            {{ errors.resource }}
          </span>
        </div>

        <div>
          <label class="label-text font-semibold" for="permissionAction">
            {{ t('admin.permissions.form.action') }} <span class="text-error">*</span>
          </label>
          <select
            id="permissionAction"
            v-model="action"
            class="select w-full"
            :class="{ 'is-invalid': errors.action }"
            :disabled="mode === 'edit'"
            :aria-required="true"
            :aria-invalid="errors.action ? 'true' : 'false'"
            :aria-describedby="errors.action ? 'permissionActionError' : undefined"
          >
            <option value="">{{ t('admin.permissions.form.select_action') }}</option>
            <option v-for="item in catalog.actions" :key="item.code" :value="item.code">
              {{ t(item.label_key) }}
            </option>
          </select>
          <span
            v-if="errors.action"
            id="permissionActionError"
            data-testid="action-error"
            role="alert"
            class="text-error text-xs mt-1 block"
          >
            {{ errors.action }}
          </span>
        </div>
      </div>

      <div>
        <span class="label-text font-semibold">{{ t('admin.permissions.form.code') }}</span>
        <p
          data-testid="code-preview"
          class="mt-1 rounded-field bg-base-200/60 px-3 py-2 font-mono text-sm"
          :class="code ? 'text-base-content' : 'text-base-content/50'"
        >
          {{ code || t('admin.permissions.form.preview_empty') }}
        </p>
      </div>

      <div
        v-if="duplicate"
        data-testid="duplicate-warning"
        role="alert"
        class="rounded-box border flex items-start gap-3 border-warning/40 bg-warning/10 px-4 py-3 text-sm text-base-content"
      >
        <span class="icon-[tabler--alert-triangle] mt-0.5 shrink-0 size-5 text-warning"></span>
        <div>
          <p class="font-semibold">{{ t('admin.permissions.form.duplicate_title') }}</p>
          <p>{{ t('admin.permissions.form.duplicate') }}</p>
        </div>
      </div>

      <div>
        <label class="label-text font-semibold" for="permissionDescription">
          {{ t('admin.permissions.form.description') }}
        </label>
        <textarea
          id="permissionDescription"
          v-model="description"
          class="textarea w-full"
          rows="3"
          :placeholder="t('admin.permissions.form.description_placeholder')"
        ></textarea>
      </div>

      <div class="flex items-center justify-end gap-3 pt-4 border-t border-base-content/10">
        <NuxtLink :to="localePath('/admin/permissions')" class="btn btn-ghost">
          {{ t('admin.permissions.form.cancel') }}
        </NuxtLink>
        <button type="submit" class="btn btn-primary" :disabled="saving">
          <span v-if="saving" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
          <span v-else class="icon-[tabler--check] size-5"></span>
          {{
            saving
              ? t('admin.permissions.form.saving')
              : mode === 'create'
                ? t('admin.permissions.form.create_submit')
                : t('admin.permissions.form.save_changes')
          }}
        </button>
      </div>
    </div>
  </form>
</template>
