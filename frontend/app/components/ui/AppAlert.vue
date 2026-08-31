<template>
  <Transition name="alert-transition">
    <div
      v-if="visible && hasContent"
      :id="alertId"
      :class="alertClasses"
      role="alert"
    >
      <span :class="iconClasses"></span>
      <p class="flex-1">
        <span v-if="title" class="text-lg font-semibold">{{ title }}: </span>
        <slot>{{ message }}</slot>
      </p>
      <button
        v-if="dismissible"
        type="button"
        class="ms-auto cursor-pointer leading-none opacity-60 hover:opacity-100 transition-opacity"
        aria-label="Close Button"
        @click="closeAlert"
      >
        <span class="icon-[tabler--x] size-5"></span>
      </button>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { useId } from 'vue'

type AlertTone = 'warning' | 'success' | 'error' | 'info'
type AlertVariant = 'solid' | 'soft' | 'outline'

const props = withDefaults(defineProps<{
  message?: string
  title?: string
  tone?: AlertTone
  variant?: AlertVariant
  icon?: string
  dismissible?: boolean
  id?: string
}>(), {
  message: '',
  title: '',
  tone: 'warning',
  variant: 'soft',
  icon: '',
  dismissible: true,
  id: '',
})

const slots = useSlots()

const visible = ref(true)
const generatedId = useId()
const alertId = computed(() => {
  if (props.id) return props.id
  return `dismiss-alert-${generatedId.replace(/[^A-Za-z0-9_-]/g, '-')}`
})

const hasContent = computed(() => {
  if (props.message) return true
  const slotContent = slots.default?.()
  if (!slotContent || slotContent.length === 0) return false
  // Check if slot has non-empty content (filter comment nodes)
  return slotContent.some((vnode: any) => {
    if (typeof vnode.children === 'string') return vnode.children.trim().length > 0
    // vnode is not a comment
    if (vnode.type && typeof vnode.type === 'symbol') return false
    return true
  })
})

const closeAlert = () => {
  visible.value = false
}

watch(
  () => props.message,
  (value) => {
    if (value) visible.value = true
  }
)

const toneClassMap: Record<AlertTone, string> = {
  warning: 'alert-warning',
  success: 'alert-success',
  error: 'alert-error',
  info: 'alert-info',
}

const toneIconMap: Record<AlertTone, string> = {
  warning: 'icon-[tabler--alert-triangle]',
  success: 'icon-[tabler--circle-check]',
  error: 'icon-[tabler--alert-circle]',
  info: 'icon-[tabler--info-circle]',
}

const variantClassMap: Record<AlertVariant, string> = {
  solid: '',
  soft: 'alert-soft',
  outline: 'alert-outline',
}

const alertClasses = computed(() => [
  'alert',
  variantClassMap[props.variant],
  toneClassMap[props.tone],
  'flex',
  'items-center',
  'gap-4',
].filter(Boolean))

const iconClasses = computed(() => [
  props.icon || toneIconMap[props.tone],
  'shrink-0',
  'size-6',
])
</script>

<style scoped>
.alert-transition-enter-active,
.alert-transition-leave-active {
  transition: all 0.3s ease-in-out;
}
.alert-transition-enter-from,
.alert-transition-leave-to {
  opacity: 0;
  transform: translateX(1.25rem);
}
</style>
