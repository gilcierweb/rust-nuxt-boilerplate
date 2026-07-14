import { test, expect } from '@playwright/test'

test.describe('Authentication Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Start from the index page
    await page.goto('/')
  })

  test('should show login link on homepage', async ({ page }) => {
    await expect(page.locator('text=Login')).toBeVisible()
  })

  test('should allow user to navigate to login page', async ({ page }) => {
    await page.click('text=Login')
    await expect(page).toHaveURL('/login')
    await expect(page.locator('h1')).toContainText('Sign in to your account')
  })

  test('should show error for invalid login', async ({ page }) => {
    await page.goto('/login')
    
    // Fill in invalid credentials
    await page.fill('input[name="email"]', 'invalid@example.com')
    await page.fill('input[name="password"]', 'wrongpassword')
    
    // Submit form
    await page.click('button[type="submit"]')
    
    // Wait for and verify error message
    const errorMessage = page.locator('.text-error')
    await expect(errorMessage).toBeVisible()
    await expect(errorMessage).toContainText('Invalid email or password')
  })

  test('should show loading state on submit', async ({ page }) => {
    await page.goto('/login')
    
    // Fill in credentials
    await page.fill('input[name="email"]', 'user@example.com')
    await page.fill('input[name="password"]', 'password123')
    
    // Submit form
    await page.click('button[type="submit"]')
    
    // Check for loading state
    const loadingSpinner = page.locator('.loading, .spinner, [aria-label="Loading"]')
    await expect(loadingSpinner).toBeAttached()
  })
})

test.describe('Protected Routes', () => {
  test('should redirect to login when accessing protected page without auth', async ({ page }) => {
    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/login')
  })
})