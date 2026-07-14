import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount } from '@vue/test-utils'
import LoginView from '@/app/pages/auth/login.vue'
import { useAuthStore } from '@/app/stores/auth'
import { createPinia, setActivePinia } from 'pinia'

describe('LoginView', () => {
  let authStore: ReturnType<typeof useAuthStore>

  beforeEach(() => {
    // Create a fresh pinia and make it active so it's automatically used
    // by any use*Store() call without having to pass it to it:
    // `useStore()` pinia.plugina
    const pinia = createPinia()
    setActivePinia(pinia)
    authStore = useAuthStore()
    
    // Mock the login method
    authStore.login = vi.fn().mockResolvedValue(undefined)
  })

  it('renders login form', () => {
    const wrapper = mount(LoginView)
    expect(wrapper.find('form').exists()).toBe(true)
    expect(wrapper.find('input[type="email"]').exists()).toBe(true)
    expect(wrapper.find('input[type="password"]').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('handles form submission', async () => {
    const wrapper = mount(LoginView)
    
    // Fill in form
    await wrapper.find('input[type="email"]').setValue('test@example.com')
    await wrapper.find('input[type="password"]').setValue('password123')
    
    // Submit form
    await wrapper.find('form').trigger('submit.prevent')
    
    // Expect login to be called with correct params
    expect(authStore.login).toHaveBeenCalledWith({
      email: 'test@example.com',
      password: 'password123',
      otp_code: undefined
    })
  })

  it('shows error when login fails', async () => {
    authStore.login.mockRejectedValueOnce(new Error('Invalid credentials'))
    
    const wrapper = mount(LoginView)
    await wrapper.find('input[type="email"]').setValue('test@example.com')
    await wrapper.find('input[type="password"]').setValue('wrongpassword')
    await wrapper.find('form').trigger('submit.prevent')
    
    // Assuming there's an error message display
    // This would need to be adjusted based on actual implementation
    expect(wrapper.text()).toContain('Error')
  })
})