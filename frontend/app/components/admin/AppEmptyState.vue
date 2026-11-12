<script setup lang="ts">
defineProps<{
  title: string
  description?: string
}>()
</script>

<template>
  <div
    class="flex flex-col items-center gap-3 rounded-box border border-dashed border-base-content/15 px-6 py-12 text-center"
    data-testid="empty-state"
  >
    <span class="icon-[tabler--database-off] size-10 text-base-content/30"></span>
    <p class="text-base font-semibold text-base-content">{{ title }}</p>
    <p
      v-if="description"
      data-testid="empty-description"
      class="max-w-md text-sm leading-relaxed text-base-content/60"
    >
      {{ description }}
    </p>
    <div v-if="$slots.action" class="mt-2 flex flex-wrap justify-center gap-2">
      <slot name="action" />
    </div>
  </div>
</template>
