import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ref } from 'vue'

// The composable reads useState as a Nuxt auto-import, through the global
// scope, and the assertions need the real reactivity: the utility shims of
// the shared setup replace ref with a plain object.
vi.unmock('vue')

const globalScope = globalThis as any

// The same keyed singleton the shared mocks hand to the app code: two calls
// of useToast must share the stack, which is the contract of the migration
// to useState.
const states = new Map<string, { value: unknown }>()
globalScope.useState = <T,>(key: string, init: () => T) => {
  if (!states.has(key)) states.set(key, ref(init()))
  return states.get(key) as { value: T }
}

const { useToast } = await import('~/composables/useToast')

describe('useToast', () => {
  beforeEach(() => {
    states.delete('app-toasts')
  })

  it('shows an info toast by default', () => {
    const { show, toasts } = useToast()
    show('Saved')

    expect(toasts.value).toHaveLength(1)
    expect(toasts.value[0]).toMatchObject({ message: 'Saved', type: 'info', duration: 4000 })
  })

  it('keeps an error on the screen longer than a success', () => {
    const { success, error, toasts } = useToast()
    success('Saved')
    error('Refused')

    expect(toasts.value.map((toast) => toast.duration)).toEqual([4000, 6000])
  })

  it('dismisses by id', () => {
    const { show, dismiss, toasts } = useToast()
    show('Saved')
    dismiss(toasts.value[0].id)

    expect(toasts.value).toHaveLength(0)
  })

  it('shares the stack between every caller', () => {
    // The migration to useState bought this: a module ref handed each caller
    // of the composable the same stack only by accident of the module
    // singleton, and a server render a fresh one per request.
    const first = useToast()
    const second = useToast()
    first.success('Saved')

    expect(second.toasts.value).toHaveLength(1)
  })
})
