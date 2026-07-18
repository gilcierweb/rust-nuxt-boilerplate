import { test, expect } from '@playwright/test'

test.describe('Login Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/login')
  })

  test('should display login form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /bem-vindo de volta/i })).toBeVisible()
    await expect(page.getByLabel(/e-mail/i)).toBeVisible()
    await expect(page.getByLabel(/senha/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /entrar/i })).toBeVisible()
  })

  test('should show links to register and forgot password', async ({ page }) => {
    await expect(page.getByRole('link', { name: /criar uma conta/i })).toHaveAttribute('href', '/auth/register')
    await expect(page.getByRole('link', { name: /esqueceu a senha/i })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should toggle password visibility', async ({ page }) => {
    const passwordInput = page.getByLabel(/senha/i)
    await expect(passwordInput).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').click()
    await expect(passwordInput).toHaveAttribute('type', 'text')

    await page.getByLabel('toggle password visibility').click()
    await expect(passwordInput).toHaveAttribute('type', 'password')
  })

  test('should show validation errors when submitting empty form', async ({ page }) => {
    await page.getByRole('button', { name: /entrar/i }).click()
    const emailField = page.getByLabel(/e-mail/i)
    const passwordField = page.getByLabel(/senha/i)
    await expect(emailField).toHaveAttribute('required', '')
    await expect(passwordField).toHaveAttribute('required', '')
  })

  test('should show error for invalid login credentials', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('nonexistent@example.com')
    await page.getByLabel(/senha/i).fill('wrongpassword')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page.getByText(/credenciais inválidas/i)).toBeVisible()
  })

  test('should display loading state during submission', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('user@example.com')
    await page.getByLabel(/senha/i).fill('password123')

    const submitButton = page.getByRole('button', { name: /entrar/i })
    await submitButton.click()
    await expect(submitButton).toBeDisabled()
  })

  test('should display 2FA OTP form when requires_otp is returned', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('2fa@example.com')
    await page.getByLabel(/senha/i).fill('correctpassword')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page.getByText(/digite o código enviado para seu e-mail/i)).toBeVisible()
    await expect(page.getByLabel(/verificar código/i)).toBeVisible()
  })

  test('should validate OTP code is exactly 6 digits', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('2fa@example.com')
    await page.getByLabel(/senha/i).fill('correctpassword')
    await page.getByRole('button', { name: /entrar/i }).click()

    const otpInput = page.getByLabel(/verificar código/i)
    await expect(otpInput).toHaveAttribute('maxlength', '6')
    await expect(otpInput).toHaveAttribute('inputmode', 'numeric')

    const verifyButton = page.getByRole('button', { name: /verificar código/i })
    await expect(verifyButton).toBeDisabled()

    await otpInput.fill('123456')
    await expect(verifyButton).toBeEnabled()
  })

  test('should show error for invalid OTP code', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('2fa@example.com')
    await page.getByLabel(/senha/i).fill('correctpassword')
    await page.getByRole('button', { name: /entrar/i }).click()

    await page.getByLabel(/verificar código/i).fill('000000')
    await page.getByRole('button', { name: /verificar código/i }).click()

    await expect(page.getByText(/código de verificação inválido/i)).toBeVisible()
  })

  test('should redirect to admin dashboard on successful admin login', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect to portal on successful non-admin login', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('user@example.com')
    await page.getByLabel(/senha/i).fill('User123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page).toHaveURL('/portal')
  })

  test('should redirect to saved returnUrl after login', async ({ page }) => {
    await page.goto('/admin/users')
    await expect(page).toHaveURL(/\/auth\/login/)

    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page).toHaveURL('/admin/users')
  })
})

test.describe('Registration Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/register')
  })

  test('should display registration form with all required fields', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /crie sua conta/i })).toBeVisible()
    await expect(page.getByLabel(/e-mail/i)).toBeVisible()
    await expect(page.getByLabel(/^senha/i)).toBeVisible()
    await expect(page.getByLabel(/confirmar senha/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /criar conta/i })).toBeVisible()
  })

  test('should show link to login page', async ({ page }) => {
    await expect(page.getByRole('link', { name: /entrar/i }).first()).toHaveAttribute('href', '/auth/login')
  })

  test('should toggle password visibility on both password fields', async ({ page }) => {
    const passwordInput = page.getByLabel(/^senha/i)
    const confirmInput = page.getByLabel(/confirmar senha/i)

    await expect(passwordInput).toHaveAttribute('type', 'password')
    await expect(confirmInput).toHaveAttribute('type', 'password')

    await page.getByLabel('toggle password visibility').first().click()
    await expect(passwordInput).toHaveAttribute('type', 'text')
    await expect(confirmInput).toHaveAttribute('type', 'text')
  })

  test('should show password strength indicator', async ({ page }) => {
    const passwordInput = page.getByLabel(/^senha/i)

    await passwordInput.fill('abc')
    await expect(page.getByText(/fraca/i)).toBeVisible()

    await passwordInput.fill('abcdef')
    await expect(page.getByText(/regular/i)).toBeVisible()

    await passwordInput.fill('Abcdef1')
    await expect(page.getByText(/boa/i)).toBeVisible()

    await passwordInput.fill('Abcdef1!')
    await expect(page.getByText(/forte/i)).toBeVisible()
  })

  test('should show password mismatch error', async ({ page }) => {
    await page.getByLabel(/^senha/i).fill('Password123!')
    await page.getByLabel(/confirmar senha/i).fill('Different123!')

    await expect(page.getByText(/as senhas não coincidem/i)).toBeVisible()
  })

  test('should disable submit button when passwords do not match', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('new@example.com')
    await page.getByLabel(/^senha/i).fill('Password123!')
    await page.getByLabel(/confirmar senha/i).fill('Different123!')
    await page.getByRole('checkbox').check()

    await expect(page.getByRole('button', { name: /criar conta/i })).toBeDisabled()
  })

  test('should require terms consent checkbox', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('new@example.com')
    await page.getByLabel(/^senha/i).fill('Password123!')
    await page.getByLabel(/confirmar senha/i).fill('Password123!')

    await expect(page.getByRole('button', { name: /criar conta/i })).toBeDisabled()

    await page.getByRole('checkbox').check()
    await expect(page.getByRole('button', { name: /criar conta/i })).toBeEnabled()
  })

  test('should show success message after registration', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('newuser@example.com')
    await page.getByLabel(/^senha/i).fill('StrongPass123!')
    await page.getByLabel(/confirmar senha/i).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.getByRole('button', { name: /criar conta/i }).click()

    await expect(page.getByText(/conta criada!/i)).toBeVisible()
    await expect(page.getByText(/enviamos um e-mail de confirmação/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /ir para login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for duplicate email registration', async ({ page }) => {
    await page.getByLabel(/e-mail/i).fill('existing@example.com')
    await page.getByLabel(/^senha/i).fill('StrongPass123!')
    await page.getByLabel(/confirmar senha/i).fill('StrongPass123!')
    await page.getByRole('checkbox').check()
    await page.getByRole('button', { name: /criar conta/i }).click()

    await expect(page.getByText(/falha no registro|já está cadastrado/i)).toBeVisible()
  })
})

test.describe('Forgot Password Page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/auth/forgot-password')
  })

  test('should display forgot password form', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /esqueceu a senha/i })).toBeVisible()
    await expect(page.getByLabel(/endereço de e-mail/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /enviar link de recuperação/i })).toBeVisible()
  })

  test('should show link back to login', async ({ page }) => {
    await expect(page.getByRole('link', { name: /entrar/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show success message after submitting email', async ({ page }) => {
    await page.getByLabel(/endereço de e-mail/i).fill('user@example.com')
    await page.getByRole('button', { name: /enviar link de recuperação/i }).click()

    await expect(page.getByText(/verifique seu e-mail/i)).toBeVisible()
    await expect(page.getByText(/enviamos um link de redefinição de senha/i)).toBeVisible()
  })

  test('should show success message even for non-existent email', async ({ page }) => {
    await page.getByLabel(/endereço de e-mail/i).fill('nonexistent@example.com')
    await page.getByRole('button', { name: /enviar link de recuperação/i }).click()

    await expect(page.getByText(/verifique seu e-mail/i)).toBeVisible()
  })
})

test.describe('Reset Password Page', () => {
  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/reset-password')

    await expect(page.getByText(/token inválido ou expirado/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /solicitar novo link/i })).toHaveAttribute('href', '/auth/forgot-password')
  })

  test('should display reset form when valid token is provided', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await expect(page.getByRole('heading', { name: /redefinir senha/i })).toBeVisible()
    await expect(page.getByLabel(/nova senha/i)).toBeVisible()
    await expect(page.getByLabel(/confirmar senha/i)).toBeVisible()
    await expect(page.getByRole('button', { name: /redefinir senha/i })).toBeVisible()
  })

  test('should toggle password visibility', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    const passwordInput = page.getByLabel(/nova senha/i)
    await expect(passwordInput).toHaveAttribute('type', 'password')

    await passwordInput.locator('..').getByRole('button').click()
    await expect(passwordInput).toHaveAttribute('type', 'text')
  })

  test('should disable submit when passwords do not match', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await page.getByLabel(/nova senha/i).fill('NewPass123!')
    await page.getByLabel(/confirmar senha/i).fill('Different123!')

    await expect(page.getByRole('button', { name: /redefinir senha/i })).toBeDisabled()
  })

  test('should show success after password reset', async ({ page }) => {
    await page.goto('/auth/reset-password?token=valid-token-123')

    await page.getByLabel(/nova senha/i).fill('NewStrongPass123!')
    await page.getByLabel(/confirmar senha/i).fill('NewStrongPass123!')
    await page.getByRole('button', { name: /redefinir senha/i }).click()

    await expect(page.getByText(/senha redefinida!/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /ir para login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid or expired token', async ({ page }) => {
    await page.goto('/auth/reset-password?token=expired-token')

    await page.getByLabel(/nova senha/i).fill('NewStrongPass123!')
    await page.getByLabel(/confirmar senha/i).fill('NewStrongPass123!')
    await page.getByRole('button', { name: /redefinir senha/i }).click()

    await expect(page.getByText(/token inválido ou expirado/i)).toBeVisible()
  })
})

test.describe('Email Confirmation Page', () => {
  test('should show loading state while confirming', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')

    await expect(page.getByText(/confirmando sua conta/i)).toBeVisible()
  })

  test('should show success after valid token confirmation', async ({ page }) => {
    await page.goto('/auth/confirm?token=valid-token')

    await expect(page.getByText(/conta confirmada!/i)).toBeVisible()
    await expect(page.getByText(/seu e-mail foi verificado/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /ir para login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error for invalid token', async ({ page }) => {
    await page.goto('/auth/confirm?token=invalid-token')

    await expect(page.getByText(/falha na confirmação/i)).toBeVisible()
    await expect(page.getByText(/inválido ou expirou/i)).toBeVisible()
    await expect(page.getByRole('link', { name: /voltar para login/i })).toHaveAttribute('href', '/auth/login')
  })

  test('should show error when accessing without token', async ({ page }) => {
    await page.goto('/auth/confirm')

    await expect(page.getByText(/falha na confirmação/i)).toBeVisible()
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
    await page.getByLabel(/e-mail/i).fill('user@example.com')
    await page.getByLabel(/senha/i).fill('User123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/portal')
  })

  test('should allow admin user to access admin routes', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await page.goto('/admin/dashboard')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should save return URL and redirect back after login', async ({ page }) => {
    await page.goto('/admin/roles/new')
    await expect(page).toHaveURL(/\/auth\/login/)

    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()

    await expect(page).toHaveURL('/admin/roles/new')
  })
})

test.describe('Authenticated User Redirect', () => {
  test('should redirect authenticated user from login to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated user from register to admin dashboard', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/e-mail/i).fill('admin@example.com')
    await page.getByLabel(/senha/i).fill('Admin123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()
    await expect(page).toHaveURL('/admin/dashboard')

    await page.goto('/auth/register')
    await expect(page).toHaveURL('/admin/dashboard')
  })

  test('should redirect authenticated non-admin user from login to portal', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByLabel(/e-mail/i).fill('user@example.com')
    await page.getByLabel(/senha/i).fill('User123!@#')
    await page.getByRole('button', { name: /entrar/i }).click()
    await expect(page).toHaveURL('/portal')

    await page.goto('/auth/login')
    await expect(page).toHaveURL('/portal')
  })
})

test.describe('Navigation Between Auth Pages', () => {
  test('should navigate from login to register', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: /criar uma conta/i }).click()
    await expect(page).toHaveURL('/auth/register')
  })

  test('should navigate from login to forgot password', async ({ page }) => {
    await page.goto('/auth/login')
    await page.getByRole('link', { name: /esqueceu a senha/i }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })

  test('should navigate from register to login', async ({ page }) => {
    await page.goto('/auth/register')
    await page.getByRole('link', { name: /entrar/i }).first().click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from forgot password to login', async ({ page }) => {
    await page.goto('/auth/forgot-password')
    await page.getByRole('link', { name: /entrar/i }).click()
    await expect(page).toHaveURL('/auth/login')
  })

  test('should navigate from reset password no-token to forgot password', async ({ page }) => {
    await page.goto('/auth/reset-password')
    await page.getByRole('link', { name: /solicitar novo link/i }).click()
    await expect(page).toHaveURL('/auth/forgot-password')
  })
})

test.describe('Homepage', () => {
  test('should load homepage', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('h1')).toContainText(/build full-stack apps/i)
  })
})
