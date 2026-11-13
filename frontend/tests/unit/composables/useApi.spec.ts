import { describe, it, expect, vi, beforeEach } from 'vitest'

// The composable reads its dependencies as the Nuxt auto-imports resolve
// them, through the global scope, so the spec installs what it needs: a
// controllable $api and the form alert spies whose contract the request
// wrapper owns.
const globalScope = globalThis as any

let apiImpl: (endpoint: string, options: unknown) => Promise<unknown> = () =>
  Promise.resolve({})

const setFormAlertMessage = vi.fn()
const clearFormAlertMessage = vi.fn()

globalScope.useNuxtApp = () => ({ $api: (endpoint: string, options: unknown) => apiImpl(endpoint, options) })
globalScope.useFormAlert = () => ({ setFormAlertMessage, clearFormAlertMessage })
globalScope.useRuntimeConfig = () => ({ public: { apiBase: '/api/v1' } })

const { useApi } = await import('~/composables/useApi')

describe('useApi', () => {
  beforeEach(() => {
    apiImpl = () => Promise.resolve({})
    setFormAlertMessage.mockClear()
    clearFormAlertMessage.mockClear()
  })

  it('makes a GET request and answers the response', async () => {
    const payload = { data: 'test data' }
    apiImpl = () => Promise.resolve(payload)

    const { get } = useApi()
    const result = await get('/test-endpoint')

    expect(result).toEqual(payload)
    expect(clearFormAlertMessage).toHaveBeenCalled()
  })

  it('makes a POST request carrying the body', async () => {
    const body = { name: 'test' }
    const spy = vi.fn(() => Promise.resolve({}))
    apiImpl = spy

    const { post } = useApi()
    await post('/test-endpoint', { body })

    // The wrapper names the method it was built for.
    expect(spy).toHaveBeenCalledWith('/test-endpoint', { body, method: 'POST' })
  })

  it('normalizes a refused request, keeps its status and alerts the form', async () => {
    // The refusal of the admin API answers this shape; the wrapper must
    // carry the status through, because the message mapper reads it.
    apiImpl = () =>
      Promise.reject({
        statusCode: 403,
        data: { error: { code: 'FORBIDDEN', message: 'Admin domain permissions required' } },
      })

    const { request } = useApi()
    const error = await request('/test-endpoint', { method: 'GET' }).catch((err) => err)

    expect(error.message).toBe('Admin domain permissions required')
    expect(error.statusCode).toBe(403)
    expect(setFormAlertMessage).toHaveBeenCalledWith('Admin domain permissions required')
  })

  it('answers an inline error body of the response as a refusal', async () => {
    apiImpl = () => Promise.resolve({ error: { message: 'Field needs attention' } })

    const { request } = useApi()

    await expect(request('/test-endpoint', { method: 'GET' })).rejects.toThrow(
      'Field needs attention',
    )
    expect(setFormAlertMessage).toHaveBeenCalledWith('Field needs attention')
  })

  it('passes a diagnostic less failure through untouched', async () => {
    // A network failure carries no api message: the wrapper must not
    // invent one, and the admin mapper answers the network sentence.
    apiImpl = () => Promise.reject(new Error('NetworkError when attempting to fetch resource.'))

    const { request } = useApi()

    await expect(request('/test-endpoint', { method: 'GET' })).rejects.toThrow(
      'NetworkError when attempting to fetch resource.',
    )
    expect(setFormAlertMessage).not.toHaveBeenCalled()
  })
})
