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

function matchesRoute(path: string, route: string) {
  return path === route || path.startsWith(`${route}/`)
}

export function isPublicRoute(path: string) {
  return PUBLIC_ROUTES.some((route) => matchesRoute(path, route))
}

export function isAuthPage(path: string) {
  return AUTH_ONLY_PAGES.some((route) => matchesRoute(path, route))
}
