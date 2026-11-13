type ToastType = 'success' | 'error' | 'warning' | 'info'

interface Toast {
  id: string
  message: string
  type: ToastType
  duration: number
}

/**
 * The toast stack of the application.
 *
 * The state lives in `useState`, not in a module ref: a ref at module scope
 * is a fresh instance on every server request, so a toast raised during a
 * server render never hydrates, and worse, one raised while a request was
 * being served bled into the renders of the next ones. `useState` is
 * per request on the server and a singleton on the client, which is what a
 * notification needs: it survives a client navigation and it serializes
 * with the payload of the render that raised it.
 *
 * The dismissal timer is scheduled only on the client: a server has no
 * business dismissing a toast for a browser that has not arrived yet.
 */
export function useToast() {
  const toasts = useState<Toast[]>('app-toasts', () => [])

  function show(message: string, type: ToastType = 'info', duration = 4000) {
    const id = Math.random().toString(36).slice(2)
    toasts.value.push({ id, type, message, duration })
    if (import.meta.client) {
      setTimeout(() => dismiss(id), duration)
    }
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
