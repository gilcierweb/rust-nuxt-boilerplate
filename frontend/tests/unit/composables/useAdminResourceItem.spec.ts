import { describe, it, expect, vi, beforeEach } from 'vitest'
import { computed, ref, watch } from 'vue'

// The mapping is a computed over the error of a real fetch rejection, so the
// reactivity and the watcher must be the real ones: the utility shims of the
// shared setup replace watch with a no-op.
vi.unmock('vue')

// The composable reads its dependencies as the Nuxt auto-imports resolve
// them, through the global scope, so the spec installs what it needs: the
// real reactivity plus the route that carries the id and a controllable api.
// The identity translation of the shared setup keeps the assertions on keys.
const globalScope = globalThis as any

globalScope.computed = computed
globalScope.ref = ref
globalScope.watch = watch

let requestImpl: (path: string, options: unknown) => Promise<unknown> = () =>
  Promise.resolve({})

globalScope.useRoute = () => ({
  path: '/admin/roles/role-1',
  params: { id: 'role-1' },
  query: {},
})
globalScope.useApi = () => ({
  request: (path: string, options: unknown) => requestImpl(path, options),
})

const { useAdminResourceItem } = await import('~/composables/useAdminResourceItem')

describe('useAdminResourceItem', () => {
  beforeEach(() => {
    requestImpl = () => Promise.resolve({})
  })

  it('maps a denial to the denial key of the viewer locale', async () => {
    requestImpl = () =>
      Promise.reject({
        statusCode: 403,
        data: { error: { code: 'FORBIDDEN', message: 'Admin domain permissions required' } },
      })

    const { error: requestError, item } = useAdminResourceItem<{ id: string }>('roles')
    await vi.waitFor(() => {
      expect(requestError.value).toBe('authorization.denied')
    })
    expect(item.value).toBeNull()
  })

  it('maps a network failure to the network sentence, never a raw diagnostic', async () => {
    requestImpl = () => Promise.reject(new Error('connect ECONNREFUSED 10.0.0.4:8080'))

    const { error: requestError } = useAdminResourceItem<{ id: string }>('roles')
    await vi.waitFor(() => {
      expect(requestError.value).toBe('common.errorExtended.network')
    })
  })

  it('answers the item and no error on a success', async () => {
    requestImpl = () => Promise.resolve({ data: { id: 'role-1' } })

    const { error: requestError, item } = useAdminResourceItem<{ id: string }>('roles')
    await vi.waitFor(() => {
      expect(item.value).toEqual({ id: 'role-1' })
    })
    expect(requestError.value).toBe('')
  })
})
