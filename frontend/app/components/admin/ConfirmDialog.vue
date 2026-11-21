<script setup lang="ts">
const { t } = useI18n()
defineProps<{
  open: boolean
  title: string
  /** The name of the target and the consequence of the confirmation. */
  message: string
  confirmLabel?: string
  cancelLabel?: string
  /** The confirmed action is running: the buttons stand down until it ends. */
  pending?: boolean
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()
</script>

<template>
  <div
    v-if="open"
    class="fixed inset-0 z-50 flex items-center justify-center bg-base-content/40 p-4"
    role="presentation"
    @click.self="emit('cancel')"
  >
    <div
      class="w-full max-w-md rounded-box border border-base-content/10 bg-base-100 p-6 shadow-lg"
      role="alertdialog"
      aria-modal="true"
      :aria-label="title"
      data-testid="confirm-dialog"
    >
      <h2 class="text-lg font-semibold text-base-content">{{ title }}</h2>
      <p class="mt-2 text-sm leading-relaxed text-base-content/70">{{ message }}</p>

      <div class="mt-6 flex items-center justify-end gap-3">
        <button
          type="button"
          class="btn btn-ghost"
          :disabled="pending"
          data-testid="confirm-cancel"
          @click="emit('cancel')"
        >
          {{ cancelLabel || t('common.cancel') }}
        </button>
        <button
          type="button"
          class="btn btn-error"
          :disabled="pending"
          data-testid="confirm-accept"
          @click="emit('confirm')"
        >
          <span v-if="pending" class="icon-[tabler--loader-2] size-4 animate-spin"></span>
          {{ confirmLabel || t('common.confirm') }}
        </button>
      </div>
    </div>
  </div>
</template>
