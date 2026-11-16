import { describe, it, expect, vi, beforeEach } from 'vitest'
import { computed, ref, unref, watch } from 'vue'

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

let routeParams: Record<string, unknown> = { id: 'role-1' }

globalScope.useRoute = () => ({
  path: '/admin/roles/role-1',
  params: routeParams,
  query: {},
})
globalScope.useApi = () => ({
  request: (path: string, options: unknown) => requestImpl(path, options),
})

// `useAsyncData` answers at once and the fetch runs in the background: the
// shim keeps the data and the error apart the way the real one does, so the
// spec can observe each branch of the returned state.
const asyncDataKeys: string[] = []
globalScope.useAsyncData = (
  key: unknown,
  handler: () => Promise<unknown>,
) => {
  // A computed key is the contract the composable uses so the payload entry
  // follows the route id; unwrapping it here mirrors what the real
  // `useAsyncData` reads.
  // A computed key is the contract the composable uses so the payload entry
  // follows the route id; `unref` mirrors what the real `useAsyncData` reads.
  asyncDataKeys.push(String(unref(key)))
  const dataRef = ref<unknown>(null)
  const errorRef = ref<unknown>(null)
  void Promise.resolve(handler()).then(
    (result) => { dataRef.value = result },
    (failure) => { errorRef.value = failure },
  )

  return {
    data: dataRef,
    pending: ref(false),
    error: errorRef,
    refresh: async () => {},
  }
}

const { useAdminResourceItem } = await import('~/composables/useAdminResourceItem')

describe('useAdminResourceItem', () => {
  beforeEach(() => {
    requestImpl = () => Promise.resolve({})
    routeParams = { id: 'role-1' }
    asyncDataKeys.length = 0
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

  it('keys the async data by the route id, so a navigation between items refetches', () => {
    useAdminResourceItem<{ id: string }>('roles', { keyPrefix: 'admin-roles-show' })

    // The server render puts the item in the payload under this key and the
    // client hydration reads it back: two items seen in sequence are two
    // payload entries, never one shared instance.
    expect(asyncDataKeys).toEqual(['admin-roles-show-role-1'])
  })

  it('fires no request at all when the route id is invalid', async () => {
    routeParams = { id: undefined }
    const requestSpy = vi.fn(() => Promise.resolve({}))
    requestImpl = requestSpy

    const { item } = useAdminResourceItem<{ id: string }>('roles')
    await vi.waitFor(() => {
      expect(item.value).toBeNull()
    })
    expect(requestSpy).not.toHaveBeenCalled()
  })
})