<template>
  <span class="badge badge-soft text-xs" :class="meta.badgeClass">
    {{ label }}
  </span>
</template>

<script setup lang="ts">
import { formatSemanticStatusLabel, resolveSemanticStatusMeta } from '~/utils/admin-status'

const props = defineProps<{
  value: unknown
  resourceSlug?: string
  columnKey: string
}>()

const { t } = useI18n()

const meta = computed(() =>
  resolveSemanticStatusMeta(props.value, {
    resourceSlug: props.resourceSlug,
    columnKey: props.columnKey,
  }),
)

const label = computed(() =>
  formatSemanticStatusLabel(props.value, t, {
    resourceSlug: props.resourceSlug,
    columnKey: props.columnKey,
  }),
)
</script>
