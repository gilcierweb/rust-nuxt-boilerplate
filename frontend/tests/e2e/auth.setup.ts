import { test as setup } from '@playwright/test'

const authFile = 'tests/e2e/.auth/user.json'

setup('set English locale', async ({ context }) => {
  await context.addCookies([
    {
      name: 'i18n_redirected',
      value: 'en',
      domain: 'localhost',
      path: '/',
    },
  ])
  await context.storageState({ path: authFile })
})
