import { describe, expect, it, vi } from 'vitest'

const globalScope = globalThis as any
const apiCalls: Array<{ url: string }> = []
let apiMock: any = {
  get: (url: string) => {
    apiCalls.push({ url })
    return Promise.resolve([{ id: '1', name: 'admin' }])
  },
}

vi.mock('~/composables/useApi', () => ({
  useApi: () => apiMock,
}))

globalScope.useApi = () => apiMock
const { useAdminLookup } = await import('~/composables/useAdminLookup')

describe('useAdminLookup', () => {
  it('loads and normalizes the requested entity', async () => {
    apiCalls.length = 0
    const lookup = useAdminLookup()
    await lookup.load('roles')
    expect(apiCalls[0]?.url).toBe('/admin/roles')
    expect(lookup.getItems('roles')).toEqual([{ id: '1', name: 'admin' }])
    expect(lookup.isLoaded('roles')).toBe(true)
  })

  it('records a message when the load fails', async () => {
    apiMock = {
      get: () => Promise.reject({ statusMessage: 'boom' }),
    }
    const lookup = useAdminLookup()
    await lookup.load('users')
    expect(lookup.getItems('users')).toEqual([])
    expect(lookup.getError('users')).toBe('boom')
  })
})
