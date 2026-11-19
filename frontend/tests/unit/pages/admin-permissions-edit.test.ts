import { describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import { mount } from '@vue/test-utils'

// The shared setup replaces `vue` and `pinia` with non reactive shims. A page
// whose template follows the data needs the real reactivity, so the watch that
// seeds the form from the loaded item runs.
vi.unmock('vue')
vi.unmock('pinia')

import { computed, nextTick, ref, toValue, watch } from 'vue'

const globalScope = globalThis as any

// Vue APIs the app resolves as auto-imports.
globalScope.ref = ref
globalScope.computed = computed
globalScope.watch = watch
globalScope.nextTick = nextTick

// A promise the test resolves when it decides, the way a slow request stays on
// the wire.
function deferred<T>() {
  let resolve!: (value: T) => void
  const promise = new Promise<T>((resolveFn) => {
    resolve = resolveFn
  })
  return { promise, resolve }
}

const itemDeferred = deferred<any>()
const catalogDeferred = deferred<any>()
const listDeferred = deferred<any>()

globalScope.useRoute = () => ({
  path: '/admin/permissions/perm-1/edit',
  params: { id: 'perm-1' },
  query: {},
  fullPath: '/admin/permissions/perm-1/edit',
  matched: [],
})
globalScope.useState = <T,>(key: string, init: () => T) => ref(init())
globalScope.useToast = () => ({ success: () => {}, error: () => {} })
globalScope.useFormAlert = () => ({
  setFormAlertMessage: () => {},
  clearFormAlertMessage: () => {},
})
globalScope.definePageMeta = () => {}
globalScope.useNuxtApp = () => ({
  $pinia: {},
  $api: async (url: string) => {
    if (url === '/admin/permissions/perm-1') return itemDeferred.promise
    if (url.endsWith('/catalog')) return catalogDeferred.promise
    if (url.startsWith('/admin/permissions')) return listDeferred.promise
    throw new Error(`unexpected request: ${url}`)
  },
})
globalScope.useRuntimeConfig = () => ({
  public: { apiBase: '/api/v1', apiDirectBase: '' },
})

// The real `useAsyncData` runs its handler the moment it is invoked. The shim
// registers every fetch and waits for `run()`, so the test decides the order
// the requests answer in: here the edited row must be allowed to answer last.
const fetches = new Map<string, { key: string; run: () => Promise<void> }>()

globalScope.useAsyncData = (
  key: unknown,
  handler: () => Promise<unknown>,
  options: Record<string, any> = {},
) => {
  const resolvedKey = String(toValue(key as any))
  const data = ref<any>(options.default?.() ?? null)
  const pending = ref(true)
  const error = ref(null)
  const run = async () => {
    try {
      data.value = await handler()
    } catch (errorValue: any) {
      error.value = errorValue
    } finally {
      pending.value = false
    }
  }
  fetches.set(resolvedKey, { key: resolvedKey, run })
  return { data, pending, error, refresh: run, execute: run }
}

// The page resolves `useApiAsyncData` and `useApi` as auto-imports, so both
// travel through the global scope, the same way the app resolves them.
globalScope.useApiAsyncData = (key: unknown, handler: () => Promise<unknown>) =>
  globalScope.useAsyncData(key, handler)

const { useApi } = await import('~/composables/useApi')
globalScope.useApi = useApi

async function flush() {
  await nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await nextTick()
}

const ITEM_KEY = 'admin-permissions-edit-perm-1'

function formFetch() {
  const entries = [...fetches.values()].filter((entry) => entry.key !== ITEM_KEY)
  expect(entries).toHaveLength(1)
  return entries[0]!
}

describe('the permission edit page', () => {
  it('does not warn about a collision with the edited row when the row answers last', async () => {
    const { default: EditPage } = await import(
      '@/app/pages/admin/permissions/[id]/edit.vue'
    )

    // The page awaits its data at setup, so it renders inside a Suspense
    // boundary the way a route renders inside the Nuxt page wrapper.
    const Harness = defineComponent({
      components: { EditPage },
      template:
        '<Suspense><EditPage /><template #fallback>loading</template></Suspense>',
    })

    const wrapper = mount(Harness, {
      global: {
        mocks: { $t: (key: string) => key },
        stubs: {
          AdminBreadcrumb: true,
          NuxtLink: true,
        },
      },
    })
    await flush()

    // The catalog and the collision list answer first, the edited row last.
    const formRun = formFetch().run()
    catalogDeferred.resolve({ resources: ['users'], actions: ['read', 'write'] })
    listDeferred.resolve({
      data: [{ code: 'users:read' }, { code: 'users:write' }],
      pagination: { total: 2 },
    })
    await formRun
    await flush()

    // The row is still on the wire, so the form waits for it.
    expect(wrapper.find('form').exists()).toBe(false)

    const itemRun = fetches.get(ITEM_KEY)!.run()
    itemDeferred.resolve({
      id: 'perm-1',
      code: 'users:read',
      name: 'Read users',
      resource: 'users',
      action: 'read',
      description: null,
    })
    await itemRun
    await flush()

    // The form carries the row, and its own code is not a collision: the
    // warning names another row, never the row being edited.
    expect(wrapper.find('#permissionName').element.value).toBe('Read users')
    expect(wrapper.find('[data-testid="duplicate-warning"]').exists()).toBe(false)
  })
})