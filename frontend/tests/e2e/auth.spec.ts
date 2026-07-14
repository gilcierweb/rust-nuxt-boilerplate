import { test, expect } from '@playwright/test'

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('should show login link on homepage', async ({ page }) => {
    await expect(page.getByText('Login')).toBeVisible()
  })

  test('should allow user to navigate to login page', async ({ page }) => {
    await page.click('text=Login')
    await expect(page).toHaveURL('/auth/login')
    await expect(page.getByText('Sign in to your account')).toBeVisible()
  })

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.goto('/auth/login')
    
    await page.fill('input[name="email"]', 'nonexistent@example.com')
    await page.fill('input[name="password"]', 'wrongpassword')
    await page.click('button[type="submit"]')
    
    await expect(page.getByText(/Invalid email or password/i)).toBeVisible()
  })

  test('should show form validation errors', async ({ page }) => {
    await page.goto('/auth/login')
    await page.click('button[type="submit"]')
    
    await expect(page.getByText(/email is required/i)).toBeVisible()
    await expect(page.getByText(/password is required/i)).toBeVisible()
  })
})

test.describe('Protected Routes', () => {
  test('should redirect to login when accessing protected page without auth', async ({ page }) => {
    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/auth/login')
  })
})