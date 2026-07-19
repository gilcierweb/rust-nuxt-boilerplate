const PUBLIC_ROUTES = [
  '/',
  '/about',
  '/contact',
  '/auth/login',
  '/auth/register',
  '/auth/forgot-password',
  '/auth/reset-password',
  '/auth/confirm',
  '/terms',
  '/privacy',
]

const AUTH_ONLY_PAGES = [
  '/auth/login',
  '/auth/register',
  '/auth/forgot-password',
  '/auth/reset-password',
  '/auth/confirm',
]

function stripLocalePrefix(path: string): string {
  return path.replace(/^\/[a-z]{2}(-[A-Z]{2})?(?=\/|$)/, '') || '/'
}

function matchesRoute(path: string, route: string) {
  const normalized = stripLocalePrefix(path)
  return normalized === route || normalized.startsWith(`${route}/`)
}

export function isPublicRoute(path: string) {
  return PUBLIC_ROUTES.some((route) => matchesRoute(path, route))
}

export function isAuthPage(path: string) {
  return AUTH_ONLY_PAGES.some((route) => matchesRoute(path, route))
}
