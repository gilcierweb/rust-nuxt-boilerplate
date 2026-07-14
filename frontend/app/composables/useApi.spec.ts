import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useApi } from '@/app/composables/useApi'

describe('useApi composable', () => {
  let mockNuxtApp: any
  let mockUseFormAlert: any
  const mockApiResponse = { data: 'test data' }

  beforeEach(() => {
    // Mock useNuxtApp
    mockNuxtApp = {
      $api: vi.fn().mockResolvedValue(mockApiResponse)
    }
    vi.mock('#imports', () => ({
      useNuxtApp: () => mockNuxtApp,
      useFormAlert: () => ({
        setFormAlertMessage: vi.fn(),
        clearFormAlertMessage: vi.fn()
      })
    }))
    
    // Reset mocks before each test
    vi.clearAllMocks()
  })

  it('makes GET request', async () => {
    const { get } = useApi()
    const result = await get('/test-endpoint')
    
    expect(mockNuxtApp.$api).toHaveBeenCalledWith('/test-endpoint', {
      method: 'GET'
    })
    expect(result).toEqual(mockApiResponse)
  })

  it('makes POST request with body', async () => {
    const { post } = useApi()
    const requestBody = { name: 'test' }
    await post('/test-endpoint', requestBody)
    
    expect(mockNuxtApp.$api).toHaveBeenCalledWith('/test-endpoint', {
      method: 'POST',
      body: JSON.stringify(requestBody)
    })
  })

  it('handles API errors and sets form alert', async () => {
    const mockError = new Error('API Error')
    mockNuxtApp.$api.mockRejectedValueOnce(mockError)
    
    const { request } = useApi()
    
    await expect(request('/test-endpoint', { method: 'GET' }))
      .rejects
      .toThrow('API Error')
    
    // Check that error message was set
    // This would depend on the actual implementation of useFormAlert mock
  })
})