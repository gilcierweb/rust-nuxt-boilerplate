import { test, expect } from '@playwright/test'
import auth from '../../i18n/locales/pt-BR/auth.json'
import common from '../../i18n/locales/pt-BR/common.json'

const a = auth.auth
const c = common.common

const SUBMIT = 'button[type="submit"]'

test.describe('Login Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/login')
  })

  test('should display login form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: a.login.title })).toBeVisible()
    await expect(page.getByLabel(a.login.email)).toBeVisible()
    await expect(page.getByLabel(a.login.password)).toBeVisible()
    await expect(page.locator(SUBMIT)).toBeVisible()
  })

  test('should show links to register and forgot password', async ({ page }) => {
    await expect(page.getByRole('link', { name: a.login.createAccount }).first()).toHaveAttribute('href', '/auth/register')
    await expect(page.getByRole('link', { name: a.login.forgotPassword })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should toggle password visibility', async ({ page }) => {
    const pw = page.getByLabel(a.login.password)
    await expect(pw).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').click()
    await expect(pw).toHaveAttribute('type', 'text')

    await page.getByLabel('toggle password visibility').click()
    await expect(pw).toHaveAttribute('type', 'password')
  })

  test('should show validation errors when submitting empty form', async ({ page }) => {
    await page.locator(SUBMIT).click()
    await expect(page.getByLabel(a.login.email)).toHaveAttribute('required', '')
    await expect(page.getByLabel(a.login.password)).toHaveAttribute('required', '')
  })

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('nonexistent@example.com')
    await page.getByLabel(a.login.password).fill('wrongpassword')
    await page.locator(SUBMIT).click()
    await expect(page.getByText(a.login.error.invalidCredentials)).toBeVisible()
  })

  test('should display loading state during submission', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('user@example.com')
    await page.getByLabel(a.login.password).fill('password123')
    const btn = page.locator(SUBMIT)
    await btn.click()
    await expect(btn).toBeDisabled()
  })

  test('should display 2FA OTP form when requires_otp is returned', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('2fa@example.com')
    await page.getByLabel(a.login.password).fill('correctpassword')
    await page.locator(SUBMIT).click()
    await expect(page.getByText(a.login.otp.label)).toBeVisible()
    await expect(page.getByLabel(a.login.otp.title)).toBeVisible()
  })

  test('should validate OTP code is exactly 6 digits', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('2fa@example.com')
    await page.getByLabel(a.login.password).fill('correctpassword')
    await page.locator(SUBMIT).click()

    const otp = page.getByLabel(a.login.otp.title)
    await expect(otp).toHaveAttribute('maxlength', '6')
    await expect(otp).toHaveAttribute('inputmode', 'numeric')

    const verifyBtn = page.getByRole('button', { name: a.login.otp.title, exact: true })
    await expect(verifyBtn).toBeDisabled()

    await otp.fill('123456')
    await expect(verifyBtn).toBeEnabled()
  })

  test('should show error for invalid OTP code', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('2fa@example.com')
    await page.getByLabel(a.login.password).fill('correctpassword')
    await page.locator(SUBMIT).click()

    await page.getByLabel(a.login.otp.title).fill('000000')
    await page.getByRole('button', { name: a.login.otp.title, exact: true }).click()
    await expect(page.getByText(a.login.otp.invalidCode)).toBeVisible()
  })

  test('should redirect to admin dashboard on successful admin login', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect to portal on successful non-admin login', async ({ page }) => {
    await page.getByLabel(a.login.email).fill('user@example.com')
    await page.getByLabel(a.login.password).fill('User123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/portal')
  })

  test('should redirect to saved returnUrl after login', async ({ page }) => {
    await page.goto('/admin/users')
    await expect(page).toHaveURL(/\/auth\/login/)
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/admin/users')
  })
})

test.describe('Registration Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/register')
  })

  test('should display registration form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: a.register.title })).toBeVisible()
    await expect(page.getByLabel(a.register.email)).toBeVisible()
    await expect(page.getByLabel(a.register.password)).toBeVisible()
    await expect(page.getByLabel(a.register.confirmPassword)).toBeVisible()
    await expect(page.locator(SUBMIT)).toBeVisible()
  })

  test('should show link to login page', async ({ page }) => {
    await expect(page.getByRole('link', { name: a.register.login }).first()).toHaveAttribute('href', '/auth/login')
  })

  test('should toggle password visibility on both password fields', async ({ page }) => {
    const pw = page.getByLabel(a.register.password)
    const confirm = page.getByLabel(a.register.confirmPassword)

    await expect(pw).toHaveAttribute('type', 'password')
    await expect(confirm).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').first().click()
    await expect(pw).toHaveAttribute('type', 'text')
    await expect(confirm).toHaveAttribute('type', 'text')
  })

  test('should show password strength indicator', async ({ page }) => {
    const pw = page.getByLabel(a.register.password)

    await pw.fill('abcdefgh')
    await expect(page.getByText(a.register.strength.weak)).toBeVisible()

    await pw.fill('Abcdefgh')
    await expect(page.getByText(a.register.strength.fair)).toBeVisible()

    await pw.fill('Abcdefgh1')
    await expect(page.getByText(a.register.strength.good)).toBeVisible()

    await pw.fill('Abcdefgh1!')
    await expect(page.getByText(a.register.strength.strong)).toBeVisible()
  })

  test('should show password mismatch error', async ({ page }) => {
    await page.getByLabel(a.register.password).fill('Password123!')
    await page.getByLabel(a.register.confirmPassword).fill('Different123!')
    await expect(page.getByText(a.register.errors.passwordMismatch)).toBeVisible()
  })

  test('should disable submit button when passwords do not match', async ({ page }) => {
    await page.getByLabel(a.register.email).fill('new@example.com')
    await page.getByLabel(a.register.password).fill('Password123!')
    await page.getByLabel(a.register.confirmPassword).fill('Different123!')
    await page.getByRole('checkbox').check()
    await expect(page.locator(SUBMIT)).toBeDisabled()
  })

  test('should require terms consent checkbox', async ({ page }) => {
    await page.getByLabel(a.register.email).fill('new@example.com')
    await page.getByLabel(a.register.password).fill('Password123!')
    await page.getByLabel(a.register.confirmPassword).fill('Password123!')
    await expect(page.locator(SUBMIT)).toBeDisabled()

    await page.getByRole('checkbox').check()
    await expect(page.locator(SUBMIT)).toBeEnabled()
  })

  test('should show success message after registration', async ({ page }) => {
    await page.getByLabel(a.register.email).fill('newuser@example.com')
    await page.getByLabel(a.register.password).fill('StrongPass123!')
    await page.getByLabel(a.register.confirmPassword).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.register.success.title)).toBeVisible()
    await expect(page.getByRole('link', { name: a.register.success.goToLogin })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for duplicate email registration', async ({ page }) => {
    await page.getByLabel(a.register.email).fill('existing@example.com')
    await page.getByLabel(a.register.password).fill('StrongPass123!')
    await page.getByLabel(a.register.confirmPassword).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.register.errors.generic)).toBeVisible()
  })
})

test.describe('Forgot Password Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/forgot-password')
  })

  test('should display forgot password form', async ({ page }) => {
    await expect(page.getByRole('heading', { name: a.forgotPassword.title })).toBeVisible()
    await expect(page.getByLabel(a.forgotPassword.email)).toBeVisible()
    await expect(page.locator(SUBMIT)).toBeVisible()
  })

  test('should show link back to login', async ({ page }) => {
    await expect(page.getByRole('link', { name: a.forgotPassword.backToLogin })).toHaveAttribute('href', '/auth/login')
  })

  test('should show success message after submitting email', async ({ page }) => {
    await page.getByLabel(a.forgotPassword.email).fill('user@example.com')
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.forgotPassword.success.title)).toBeVisible()
    await expect(page.getByText(a.forgotPassword.success.message)).toBeVisible()
  })

  test('should show success message even for non-existent email', async ({ page }) => {
    await page.getByLabel(a.forgotPassword.email).fill('nonexistent@example.com')
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.forgotPassword.success.title)).toBeVisible()
  })
})

test.describe('Reset Password Page', () => {
  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/reset-password')
    await expect(page.getByText(a.resetPassword.error.invalidToken)).toBeVisible()
    await expect(page.getByRole('link', { name: a.resetPassword.error.requestNew })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should display reset form when valid token is provided', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')
    await expect(page.getByRole('heading', { name: a.resetPassword.title })).toBeVisible()
    await expect(page.getByLabel(a.resetPassword.newPassword)).toBeVisible()
    await expect(page.getByLabel(a.resetPassword.confirmPassword)).toBeVisible()
    await expect(page.locator(SUBMIT)).toBeVisible()
  })

  test('should toggle password visibility', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')
    const pw = page.getByLabel(a.resetPassword.newPassword)
    await expect(pw).toHaveAttribute('type', 'password')
    await pw.locator('..').getByRole('button').click()
    await expect(pw).toHaveAttribute('type', 'text')
  })

  test('should disable submit when passwords do not match', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')
    await page.getByLabel(a.resetPassword.newPassword).fill('NewPass123!')
    await page.getByLabel(a.resetPassword.confirmPassword).fill('Different123!')
    await expect(page.locator(SUBMIT)).toBeDisabled()
  })

  test('should show success after password reset', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')
    await page.getByLabel(a.resetPassword.newPassword).fill('NewStrongPass123!')
    await page.getByLabel(a.resetPassword.confirmPassword).fill('NewStrongPass123!')
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.resetPassword.success.title)).toBeVisible()
    await expect(page.getByRole('link', { name: a.resetPassword.success.login })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid or expired token', async ({ page }) => {
    await page.goto('/auth/reset-password?token=expired-token')
    await page.getByLabel(a.resetPassword.newPassword).fill('NewStrongPass123!')
    await page.getByLabel(a.resetPassword.confirmPassword).fill('NewStrongPass123!')
    await page.locator(SUBMIT).click()

    await expect(page.getByText(a.resetPassword.error.invalidToken)).toBeVisible()
  })
})

test.describe('Email Confirmation Page', () => {
  test('should show loading state while confirming', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')
    await expect(page.getByText(a.confirm.loading)).toBeVisible()
  })

  test('should show success after valid token confirmation', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')
    await expect(page.getByText(a.confirm.success.title)).toBeVisible()
    await expect(page.getByText(a.confirm.success.message)).toBeVisible()
    await expect(page.getByRole('link', { name: a.confirm.success.login })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid token', async ({ page }) => {
    await page.goto('/auth/confirm?token=invalid-token')
    await expect(page.getByText(a.confirm.error.title)).toBeVisible()
    await expect(page.getByText(a.confirm.error.message)).toBeVisible()
    await expect(page.getByRole('link', { name: a.confirm.error.backToLogin })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/confirm')
    await expect(page.getByText(a.confirm.error.title)).toBeVisible()
  })
})

test.describe('Protected Routes', () => {
  test('should redirect unauthenticated user to login for admin routes', async ({ page }) => {
    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL(/\/auth\/login/)
  })

  test('should redirect unauthenticated user to login for portal routes', async ({ page }) => {
    await page.goto('/portal/dashboard')
    await expect(page).toHaveURL(/\/auth\/login/)
  })

  test('should redirect unauthenticated user to login for admin users page', async ({ page }) => {
    await page.goto('/admin/users')
    await expect(page).toHaveURL(/\/auth\/login/)
  })

  test('should redirect unauthenticated user to login for admin roles page', async ({ page }) => {
    await page.goto('/admin/roles')
    await expect(page).toHaveURL(/\/auth\/login/)
  })

  test('should redirect non-admin user from admin routes to portal', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(a.login.email).fill('user@example.com')
    await page.getByLabel(a.login.password).fill('User123!@#')
    await page.locator(SUBMIT).click()
    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/portal')
  })

  test('should allow admin user to access admin routes', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should save return URL and redirect back after login', async ({ page }) => {
    await page.goto('/admin/roles/new')
    await expect(page).toHaveURL(/\/auth\/login/)
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/admin/roles/new')
  })
})

test.describe('Authenticated User Redirect', () => {
  test('should redirect authenticated user from login to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated user from register to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(a.login.email).fill('admin@example.com')
    await page.getByLabel(a.login.password).fill('Admin123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/register')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated non-admin user from login to portal', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(a.login.email).fill('user@example.com')
    await page.getByLabel(a.login.password).fill('User123!@#')
    await page.locator(SUBMIT).click()
    await expect(page).toHaveURL('/portal')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/portal')
  })
})

test.describe('Navigation Between Auth Pages', () => {
  test('should navigate from login to register', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: a.login.createAccount }).first().click()
    await expect(page).toHaveURL('/auth/register')
  })

  test('should navigate from login to forgot password', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: a.login.forgotPassword }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })

  test('should navigate from register to login', async ({ page }) => {
    await page.goto('/auth/register')
    await page.getByRole('link', { name: a.register.login }).first().click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from forgot password to login', async ({ page }) => {
    await page.goto('/auth/forgot-password')
    await page.getByRole('link', { name: a.forgotPassword.backToLogin }).click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from reset password no-token to forgot password', async ({ page }) => {
    await page.goto('/auth/reset-password')
    await page.getByRole('link', { name: a.resetPassword.error.requestNew }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })
})

test.describe('Homepage', () => {
  test('should load homepage', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1')).toContainText(/build full-stack apps/i)
  })
})
