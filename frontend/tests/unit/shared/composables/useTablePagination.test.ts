import { describe, expect, it, vi } from 'vitest'

// The shared setup replaces `vue` and `pinia` with non reactive shims. A
// composable that drives a real `useAsyncData` needs the real reactivity.
vi.unmock('vue')
vi.unmock('pinia')

import { computed, nextTick, ref, watch } from 'vue'

const apiCalls: Array<{ url: string; query: Record<string, unknown> }> = []

// Nuxt auto-imports are not rewritten outside the Nuxt pipeline, so the
// composable resolves every bare identifier from the global scope.
const globalScope = globalThis as any
globalScope.ref = ref
globalScope.computed = computed
globalScope.watch = watch
globalScope.nextTick = nextTick
globalScope.useRuntimeConfig = () => ({ public: { apiBase: '/api/v1' } })
globalScope.useNuxtApp = () => ({
  $api: async (url: string, options: { query?: Record<string, unknown> }) => {
    apiCalls.push({ url, query: options?.query ?? {} })
    return { data: [], pagination: null }
  },
})

// `useAsyncData` is synchronous in the real composable: the object answers at
// once and the fetch runs in the background. An async shim would hand the
// caller a Promise, and the destructuring would read every field as undefined.
globalScope.useAsyncData = (
  _key: unknown,
  handler: () => Promise<unknown>,
) => {
  const dataRef = ref<unknown>({ data: [], pagination: null })
  void Promise.resolve(handler()).then((result) => {
    dataRef.value = result
  })

  return {
    data: dataRef,
    pending: ref(false),
    error: ref(null),
    refresh: async () => {},
    execute: async () => {
      dataRef.value = await handler()
    },
  }
}

import { useTablePagination } from '@/app/composables/useTablePagination'

/** Flushes the background fetch the shim started. */
async function settle() {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

/**
 * A query parameter with no value.
 *
 * `$fetch` serializes `null` as a bare key (`?sort_by`), and the backend reads
 * that as an empty string. An empty string is not part of the sort allowlist,
 * so every list answers 422 until the admin picks a column, and the only way
 * to see the shape of the request is the keys it carries.
 */
describe('the query of the admin lists', () => {
  it('omits the sort keys while no column is chosen', async () => {
    apiCalls.length = 0

    useTablePagination(() => ({ url: '/admin/roles' }))
    await settle()

    expect(apiCalls).toHaveLength(1)
    expect(apiCalls[0]!.url).toContain('/admin/roles')
    expect(apiCalls[0]!.query).toMatchObject({ page: 1, per_page: 10 })
    expect('sort_by' in apiCalls[0]!.query).toBe(false)
    expect('sort_dir' in apiCalls[0]!.query).toBe(false)
    expect('q' in apiCalls[0]!.query).toBe(false)
  })

  it('sends the chosen column and its direction', async () => {
    apiCalls.length = 0

    const table = useTablePagination(() => ({ url: '/admin/roles' }))
    await settle()

    table.setSorting([{ id: 'name', desc: false }])
    await settle()

    const query = apiCalls.at(-1)!.query
    expect(query['sort_by']).toBe('name')
    expect(query['sort_dir']).toBe('asc')
  })
})
