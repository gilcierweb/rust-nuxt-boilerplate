import { test, expect } from '@playwright/test'
import { createRequire } from 'module'

// The assertions name the locale keys, not the sentences, the same way the
// login spec does: the suite stays independent of the locale files.
const require = createRequire(import.meta.url)
const authorization = require('../../i18n/locales/pt-BR/authorization.json').authorization

/**
 * The denial of the admin domain, end to end.
 *
 * The suite needs the stack the CI workflow assembles: the backend on 8080
 * with the seeded database (the rotation of the seed gives user06 the
 * customer role, the self scoped one, the subject that opens no admin
 * scope) and the frontend served from the node-server build.
 */
const CUSTOMER_EMAIL = 'user06@example.com'
const SEED_PASSWORD = 'password123'
const API_BASE = process.env.E2E_API_BASE ?? 'http://localhost:8080/api/v1'

test.describe('Admin denial', () => {
  test.beforeEach(async ({ page, request, context }) => {
    // The session is established against the API, the way a real login does,
    // so the cookies the middleware reads exist before the first navigation.
    const login = await request.post(`${API_BASE}/auth/login`, {
      data: { email: CUSTOMER_EMAIL, password: SEED_PASSWORD },
    })
    expect(login.ok(), 'the seeded customer must be able to log in').toBeTruthy()

    const storage = await request.storageState()
    await context.addCookies(
      storage.cookies.map((cookie) => ({
        name: cookie.name,
        value: cookie.value,
        domain: cookie.domain,
        path: cookie.path,
      })),
    )
  })

  test('a subject without the admin authority lands on the portal with the denial toast', async ({
    page,
  }) => {
    await page.goto('/admin/users')
    await page.waitForLoadState('networkidle')

    // The middleware answers the redirect straight to the dashboard. The
    // marker rides the query and is spent on arrival, so by the time the
    // page is idle the address bar is already clean: the proof that it
    // traveled is the toast, not the URL.
    await expect(page).toHaveURL(/\/portal\/dashboard$/)

    // The toast says why, in the locale of the viewer.
    const toast = page.locator('[role="alert"]').first()
    await expect(toast).toBeVisible()
    await expect(toast).toContainText(authorization.denied)
  })

  test('a refresh of the landing does not repeat the notice', async ({ page }) => {
    await page.goto('/admin/users')
    await page.waitForLoadState('networkidle')
    await expect(page.locator('[role="alert"]').first()).toBeVisible()

    await page.reload()
    await page.waitForLoadState('networkidle')

    await expect(page.locator('[role="alert"]')).toHaveCount(0)
  })
})
