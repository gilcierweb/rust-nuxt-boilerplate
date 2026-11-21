import { describe, expect, it, vi } from 'vitest'

const apiCalls: Array<{ method: string; url: string; options?: any }> = []
let apiMock: any = {
  get: (url: string, options?: any) => {
    apiCalls.push({ method: 'GET', url, options })
    return Promise.resolve({ data: [], pagination: { page: 1, per_page: 10, total: 0, total_pages: 0, has_next: false, has_prev: false } })
  },
  post: (url: string, options?: any) => {
    apiCalls.push({ method: 'POST', url, options })
    return Promise.resolve({ id: '1' })
  },
  patch: (url: string, options?: any) => {
    apiCalls.push({ method: 'PATCH', url, options })
    return Promise.resolve({ id: '1' })
  },
  put: (url: string, options?: any) => {
    apiCalls.push({ method: 'PUT', url, options })
    return Promise.resolve({ updated: 1 })
  },
  delete: (url: string) => {
    apiCalls.push({ method: 'DELETE', url })
    return Promise.resolve()
  },
}

vi.mock('~/composables/useApi', () => ({
  useApi: () => apiMock,
}))

const { useAdminRbac } = await import('~/composables/useAdminRbac')

describe('useAdminRbac', () => {
  it('lists roles with the non-empty query only', async () => {
    apiCalls.length = 0
    const { roles } = useAdminRbac()
    await roles.list({ page: 2, q: 'admin', sort_dir: '' })
    expect(apiCalls[0]).toMatchObject({
      method: 'GET',
      url: '/admin/roles',
      options: { query: { page: 2, q: 'admin' } },
    })
  })

  it('posts a permission payload', async () => {
    apiCalls.length = 0
    const { permissions } = useAdminRbac()
    await permissions.create({ name: 'read', resource: 'users', action: 'read' })
    expect(apiCalls[0]).toMatchObject({
      method: 'POST',
      url: '/admin/permissions',
      options: { body: { name: 'read', resource: 'users', action: 'read' } },
    })
  })

  it('unwraps role permissions', async () => {
    apiMock = {
      get: () => Promise.resolve({ permissions: [{ id: '1' }] }),
      post: () => Promise.resolve({}),
      patch: () => Promise.resolve({}),
      put: () => Promise.resolve({}),
      delete: () => Promise.resolve(),
    }
    const { rolePermissions } = useAdminRbac()
    await expect(rolePermissions.get('role-1')).resolves.toEqual([{ id: '1' }])
  })
})
