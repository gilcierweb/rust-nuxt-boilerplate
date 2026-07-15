import { test, expect } from '@playwright/test'

test.describe('Login Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/login')
  })

  test('should display login form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /welcome back/i })).toBeVisible()
    await expect(page.getByLabel(/email/i)).toBeVisible()
    await expect(page.getByLabel(/password/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /sign in/i })).toBeVisible()
  })

  test('should show links to register and forgot password', async ({ page }) => {
    await expect(page.getByRole('link', { name: /create an account/i })).toHaveAttribute('href', '/auth/register')
    await expect(page.getByRole('link', { name: /forgot password/i })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should toggle password visibility', async ({ page }) => {
    const passwordInput = page.getByLabel(/password/i)
    await expect(passwordInput).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').click()
    await expect(passwordInput).toHaveAttribute('type', 'text')

    await page.getByLabel('toggle password visibility').click()
    await expect(passwordInput).toHaveAttribute('type', 'password')
  })

  test('should show validation errors when submitting empty form', async ({ page }) => {
    await page.getByRole('button', { name: /sign in/i }).click()
    const emailField = page.getByLabel(/email/i)
    const passwordField = page.getByLabel(/password/i)
    await expect(emailField).toHaveAttribute('required', '')
    await expect(passwordField).toHaveAttribute('required', '')
  })

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.getByLabel(/email/i).fill('nonexistent@example.com')
    await page.getByLabel(/password/i).fill('wrongpassword')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page.getByText(/invalid credentials/i)).toBeVisible()
  })

  test('should display loading state during submission', async ({ page }) => {
    await page.getByLabel(/email/i).fill('user@example.com')
    await page.getByLabel(/password/i).fill('password123')

    const submitButton = page.getByRole('button', { name: /sign in/i })
    await submitButton.click()
    await expect(submitButton).toBeDisabled()
  })

  test('should display 2FA OTP form when requires_otp is returned', async ({ page }) => {
    await page.getByLabel(/email/i).fill('2fa@example.com')
    await page.getByLabel(/password/i).fill('correctpassword')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page.getByText(/enter the code sent to your email/i)).toBeVisible()
    await expect(page.getByLabel(/verify code/i)).toBeVisible()
  })

  test('should validate OTP code is exactly 6 digits', async ({ page }) => {
    await page.getByLabel(/email/i).fill('2fa@example.com')
    await page.getByLabel(/password/i).fill('correctpassword')
    await page.getByRole('button', { name: /sign in/i }).click()

    const otpInput = page.getByLabel(/verify code/i)
    await expect(otpInput).toHaveAttribute('maxlength', '6')
    await expect(otpInput).toHaveAttribute('inputmode', 'numeric')

    const verifyButton = page.getByRole('button', { name: /verify code/i })
    await expect(verifyButton).toBeDisabled()

    await otpInput.fill('123456')
    await expect(verifyButton).toBeEnabled()
  })

  test('should show error for invalid OTP code', async ({ page }) => {
    await page.getByLabel(/email/i).fill('2fa@example.com')
    await page.getByLabel(/password/i).fill('correctpassword')
    await page.getByRole('button', { name: /sign in/i }).click()

    await page.getByLabel(/verify code/i).fill('000000')
    await page.getByRole('button', { name: /verify code/i }).click()

    await expect(page.getByText(/invalid verification code/i)).toBeVisible()
  })

  test('should redirect to admin dashboard on successful admin login', async ({ page }) => {
    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect to portal on successful non-admin login', async ({ page }) => {
    await page.getByLabel(/email/i).fill('user@example.com')
    await page.getByLabel(/password/i).fill('User123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page).toHaveURL('/portal')
  })

  test('should redirect to saved returnUrl after login', async ({ page }) => {
    await page.goto('/admin/users')
    await expect(page).toHaveURL(/\/auth\/login/)

    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page).toHaveURL('/admin/users')
  })
})

test.describe('Registration Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/register')
  })

  test('should display registration form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /create your account/i })).toBeVisible()
    await expect(page.getByLabel(/email/i)).toBeVisible()
    await expect(page.getByLabel(/^password/i)).toBeVisible()
    await expect(page.getByLabel(/confirm password/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /create account/i })).toBeVisible()
  })

  test('should show link to login page', async ({ page }) => {
    await expect(page.getByRole('link', { name: /log in/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should toggle password visibility on both password fields', async ({ page }) => {
    const passwordInput = page.getByLabel(/^password/i)
    const confirmInput = page.getByLabel(/confirm password/i)

    await expect(passwordInput).toHaveAttribute('type', 'password')
    await expect(confirmInput).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').first().click()
    await expect(passwordInput).toHaveAttribute('type', 'text')
    await expect(confirmInput).toHaveAttribute('type', 'text')
  })

  test('should show password strength indicator', async ({ page }) => {
    const passwordInput = page.getByLabel(/^password/i)

    await passwordInput.fill('abc')
    await expect(page.getByText(/weak/i)).toBeVisible()

    await passwordInput.fill('abcdef')
    await expect(page.getByText(/fair/i)).toBeVisible()

    await passwordInput.fill('Abcdef1')
    await expect(page.getByText(/good/i)).toBeVisible()

    await passwordInput.fill('Abcdef1!')
    await expect(page.getByText(/strong/i)).toBeVisible()
  })

  test('should show password mismatch error', async ({ page }) => {
    await page.getByLabel(/^password/i).fill('Password123!')
    await page.getByLabel(/confirm password/i).fill('Different123!')

    await expect(page.getByText(/passwords don't match/i)).toBeVisible()
  })

  test('should disable submit button when passwords do not match', async ({ page }) => {
    await page.getByLabel(/email/i).fill('new@example.com')
    await page.getByLabel(/^password/i).fill('Password123!')
    await page.getByLabel(/confirm password/i).fill('Different123!')
    await page.getByRole('checkbox').check()

    await expect(page.getByRole('button', { name: /create account/i })).toBeDisabled()
  })

  test('should require terms consent checkbox', async ({ page }) => {
    await page.getByLabel(/email/i).fill('new@example.com')
    await page.getByLabel(/^password/i).fill('Password123!')
    await page.getByLabel(/confirm password/i).fill('Password123!')

    await expect(page.getByRole('button', { name: /create account/i })).toBeDisabled()

    await page.getByRole('checkbox').check()
    await expect(page.getByRole('button', { name: /create account/i })).toBeEnabled()
  })

  test('should show success message after registration', async ({ page }) => {
    await page.getByLabel(/email/i).fill('newuser@example.com')
    await page.getByLabel(/^password/i).fill('StrongPass123!')
    await page.getByLabel(/confirm password/i).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.getByRole('button', { name: /create account/i }).click()

    await expect(page.getByText(/account created!/i)).toBeVisible()
    await expect(page.getByText(/we sent a confirmation email/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /go to login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for duplicate email registration', async ({ page }) => {
    await page.getByLabel(/email/i).fill('existing@example.com')
    await page.getByLabel(/^password/i).fill('StrongPass123!')
    await page.getByLabel(/confirm password/i).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.getByRole('button', { name: /create account/i }).click()

    await expect(page.getByText(/registration failed|already exists/i)).toBeVisible()
  })
})

test.describe('Forgot Password Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/forgot-password')
  })

  test('should display forgot password form', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /forgot password/i })).toBeVisible()
    await expect(page.getByLabel(/email address/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /send reset link/i })).toBeVisible()
  })

  test('should show link back to login', async ({ page }) => {
    await expect(page.getByRole('link', { name: /log in/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show success message after submitting email', async ({ page }) => {
    await page.getByLabel(/email address/i).fill('user@example.com')
    await page.getByRole('button', { name: /send reset link/i }).click()

    await expect(page.getByText(/check your email/i)).toBeVisible()
    await expect(page.getByText(/we sent a password reset link/i)).toBeVisible()
  })

  test('should show success message even for non-existent email', async ({ page }) => {
    await page.getByLabel(/email address/i).fill('nonexistent@example.com')
    await page.getByRole('button', { name: /send reset link/i }).click()

    await expect(page.getByText(/check your email/i)).toBeVisible()
  })
})

test.describe('Reset Password Page', () => {
  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/reset-password')

    await expect(page.getByText(/invalid or expired token/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /request new link/i })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should display reset form when valid token is provided', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await expect(page.getByRole('heading', { name: /reset password/i })).toBeVisible()
    await expect(page.getByLabel(/new password/i)).toBeVisible()
    await expect(page.getByLabel(/confirm password/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /reset password/i })).toBeVisible()
  })

  test('should toggle password visibility', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    const passwordInput = page.getByLabel(/new password/i)
    await expect(passwordInput).toHaveAttribute('type', 'password')

    await passwordInput.locator('..').getByRole('button').click()
    await expect(passwordInput).toHaveAttribute('type', 'text')
  })

  test('should disable submit when passwords do not match', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await page.getByLabel(/new password/i).fill('NewPass123!')
    await page.getByLabel(/confirm password/i).fill('Different123!')

    await expect(page.getByRole('button', { name: /reset password/i })).toBeDisabled()
  })

  test('should show success after password reset', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await page.getByLabel(/new password/i).fill('NewStrongPass123!')
    await page.getByLabel(/confirm password/i).fill('NewStrongPass123!')
    await page.getByRole('button', { name: /reset password/i }).click()

    await expect(page.getByText(/password reset!/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /go to login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid or expired token', async ({ page }) => {
    await page.goto('/auth/reset-password?token=expired-token')

    await page.getByLabel(/new password/i).fill('NewStrongPass123!')
    await page.getByLabel(/confirm password/i).fill('NewStrongPass123!')
    await page.getByRole('button', { name: /reset password/i }).click()

    await expect(page.getByText(/invalid or expired token/i)).toBeVisible()
  })
})

test.describe('Email Confirmation Page', () => {
  test('should show loading state while confirming', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')

    await expect(page.getByText(/confirming your account/i)).toBeVisible()
  })

  test('should show success after valid token confirmation', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')

    await expect(page.getByText(/account confirmed!/i)).toBeVisible()
    await expect(page.getByText(/your email has been verified/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /go to login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid token', async ({ page }) => {
    await page.goto('/auth/confirm?token=invalid-token')

    await expect(page.getByText(/confirmation failed/i)).toBeVisible()
    await expect(page.getByText(/invalid or has expired/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /back to login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/confirm')

    await expect(page.getByText(/confirmation failed/i)).toBeVisible()
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
    await page.getByLabel(/email/i).fill('user@example.com')
    await page.getByLabel(/password/i).fill('User123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/portal')
  })

  test('should allow admin user to access admin routes', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should save return URL and redirect back after login', async ({ page }) => {
    await page.goto('/admin/roles/new')
    await expect(page).toHaveURL(/\/auth\/login/)

    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()

    await expect(page).toHaveURL('/admin/roles/new')
  })
})

test.describe('Authenticated User Redirect', () => {
  test('should redirect authenticated user from login to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated user from register to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/email/i).fill('admin@example.com')
    await page.getByLabel(/password/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/register')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated non-admin user from login to portal', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/email/i).fill('user@example.com')
    await page.getByLabel(/password/i).fill('User123!@#')
    await page.getByRole('button', { name: /sign in/i }).click()
    await expect(page).toHaveURL('/portal')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/portal')
  })
})

test.describe('Navigation Between Auth Pages', () => {
  test('should navigate from login to register', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: /create an account/i }).click()
    await expect(page).toHaveURL('/auth/register')
  })

  test('should navigate from login to forgot password', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: /forgot password/i }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })

  test('should navigate from register to login', async ({ page }) => {
    await page.goto('/auth/register')
    await page.getByRole('link', { name: /log in/i }).click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from forgot password to login', async ({ page }) => {
    await page.goto('/auth/forgot-password')
    await page.getByRole('link', { name: /log in/i }).click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from reset password no-token to forgot password', async ({ page }) => {
    await page.goto('/auth/reset-password')
    await page.getByRole('link', { name: /request new link/i }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })
})

test.describe('Homepage', () => {
  test('should display login link on homepage', async ({ page }) => {
    await page.goto('/')
    await expect(page.getByRole('link', { name: /login/i })).toBeVisible()
  })

  test('should navigate to login page from homepage', async ({ page }) => {
    await page.goto('/')
    await page.getByRole('link', { name: /login/i }).first().click()
    await expect(page).toHaveURL('/auth/login')
  })
})
