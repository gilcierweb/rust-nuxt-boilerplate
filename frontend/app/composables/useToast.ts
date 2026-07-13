type ToastType = 'success' | 'error' | 'warning' | 'info'

interface Toast {
  id: string
  type: ToastType
  message: string
  duration: number
}

const toasts = ref<Toast[]>([])

export function useToast() {
  function show(message: string, type: ToastType = 'info', duration = 4000) {
    const id = Math.random().toString(36).slice(2)
    toasts.value.push({ id, type, message, duration })
    setTimeout(() => dismiss(id), duration)
  }

  function dismiss(id: string) {
    const idx = toasts.value.findIndex((t) => t.id === id)
    if (idx > -1) toasts.value.splice(idx, 1)
  }

  const success = (msg: string) => show(msg, 'success')
  const error = (msg: string) => show(msg, 'error', 6000)
  const warning = (msg: string) => show(msg, 'warning')
  const info = (msg: string) => show(msg, 'info')

  return { toasts, show, dismiss, success, error, warning, info }
}
