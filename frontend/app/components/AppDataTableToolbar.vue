<template>
  <div class="mb-5 flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
    <div v-if="showTotal" class="rounded-box bg-base-200/70 px-4 py-3">
      <p class="text-xs font-semibold uppercase tracking-[0.18em] text-base-content/45">{{ totalLabel }}</p>
      <p class="mt-2 text-2xl font-semibold text-base-content">{{ total }}</p>
    </div>

    <div class="flex flex-wrap items-center gap-2">
      <slot name="left" />

      <label
        v-if="enableGlobalFilter"
        class="flex min-w-72 items-center gap-3 rounded-box border border-base-content/10 bg-base-200/70 px-3 py-2.5"
      >
        <span class="icon-[tabler--search] size-4 text-base-content/55"></span>
        <input
          :value="modelValue"
          type="search"
          class="w-full bg-transparent text-sm outline-none placeholder:text-base-content/45"
          :placeholder="searchPlaceholder"
          @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <slot name="right" />

      <span v-if="loading" class="loading loading-spinner loading-sm text-primary" />

      <button
        v-if="showRefresh"
        type="button"
        class="btn btn-soft"
        :disabled="loading"
        @click="$emit('refresh')"
      >
        <span class="icon-[tabler--refresh] size-4.5"></span>
        {{ refreshLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  modelValue?: string
  searchPlaceholder?: string
  totalLabel?: string
  total?: number | string
  showTotal?: boolean
  enableGlobalFilter?: boolean
  showRefresh?: boolean
  refreshLabel?: string
  loading?: boolean
}>(), {
  modelValue: '',
  searchPlaceholder: '...',
  totalLabel: 'Total',
  total: 0,
  showTotal: true,
  enableGlobalFilter: true,
  showRefresh: false,
  refreshLabel: 'Atualizar',
  loading: false,
})

defineEmits<{
  'update:modelValue': [value: string]
  refresh: []
}>()
</script>
