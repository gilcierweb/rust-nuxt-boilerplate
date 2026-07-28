<template>
  <div class="py-10 text-center text-base-content/55">
    <template v-if="state === 'loading'">
      <span class="icon-[tabler--loader-2] mr-2 inline-block size-6 animate-spin"></span>
      {{ label }}
    </template>
    <template v-else-if="state === 'error'">
      <span class="icon-[tabler--alert-circle] mr-2 inline-block size-6 text-error"></span>
      <span class="text-error">{{ label }}</span>
    </template>
    <template v-else>
      <span class="icon-[tabler--database-exclamation mr-2 inline-block size-6]"></span>
      {{ label }}
    </template>
  </div>
</template>

<script setup lang="ts">
type DataState = 'loading' | 'error' | 'empty'

withDefaults(defineProps<{
  state: DataState
  label: string
}>(), {
  state: 'empty',
})
</script>
