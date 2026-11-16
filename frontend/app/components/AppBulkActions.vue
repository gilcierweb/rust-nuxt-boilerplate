<template>
  <div
    v-if="count > 0"
    class="mb-4 flex flex-col gap-3 rounded-box border border-primary/20 bg-primary/10 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
    role="region"
    :aria-label="$t('admin.common.bulkActions.regionLabel')"
    data-testid="bulk-actions"
  >
    <div class="flex items-center gap-3">
      <span class="rounded-full bg-primary/10 px-3 py-1.5 text-sm font-semibold text-primary">
        {{ count }}
      </span>
      <span class="text-sm text-base-content/70">{{ label }}</span>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <slot />
      <button
        type="button"
        class="btn btn-soft btn-sm transition-none"
        data-testid="bulk-clear"
        @click="emit('clear')"
      >
        <span class="icon-[tabler--x] size-4"></span>
        {{ $t('admin.common.bulkActions.clear') }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    /** How many rows the selection holds right now. */
    count: number
    /** The sentence beside the count, translated by the caller. */
    label: string
  }>(),
  {
    count: 0,
    label: '',
  },
)

const emit = defineEmits<{
  clear: []
}>()

// Escape discards the selection. The bar only exists while rows are
// selected, so the listener lives exactly as long as it can act.
useEventListener(window, 'keydown', (event: KeyboardEvent) => {
  if (event.key === 'Escape') emit('clear')
})
</script>