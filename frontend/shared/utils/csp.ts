/**
 * Content-Security-Policy for the Nuxt app (single source of truth).
 *
 * `nuxt.security` merges nothing: assigning `security.headers.contentSecurityPolicy`
 * replaces the whole policy, so every directive this app relies on has to be
 * spelled out here or it silently disappears. The default policy that
 * `nuxt-security` would otherwise apply is therefore reproduced (and extended)
 * rather than partially overridden.
 *
 * Used by `nuxt.config.ts`.
 *
 * Two deliberate deviations from a strict policy, both documented:
 * - `script-src 'unsafe-inline'`: Nuxt's SSR payload is an inline script, and
 *   the app renders no user HTML (no `v-html`, no `innerHTML`), so the
 *   directive is a compatibility requirement rather than an XSS gap.
 * - `style-src 'unsafe-inline'`: Tailwind and FlyonUI write inline styles.
 *   Styles cannot execute, so the exposure is limited to layout defacement.
 */

/** Origins the dev server may talk to (Vite HMR and the local API).
 * Only included outside production: in production every call is same-origin,
 * including the WebSocket upgrade. */
const DEV_CONNECT_SOURCES = [
  "'self'",
  'ws://localhost:*',
  'wss://localhost:*',
]

export function contentSecurityPolicy(isDev: boolean) {
  return {
    'default-src': ["'self'"],
    'base-uri': ["'none'"],
    'object-src': ["'none'"],
    'form-action': ["'self'"],
    'frame-ancestors': ["'self'"],
    'img-src': ["'self'", 'data:', 'blob:'],
    'font-src': ["'self'", 'fonts.gstatic.com', 'data:'],
    'style-src': ["'self'", "'unsafe-inline'", 'fonts.googleapis.com'],
    'script-src': ["'self'", "'unsafe-inline'"],
    'script-src-attr': ["'none'"],
    'connect-src': isDev ? DEV_CONNECT_SOURCES : ["'self'"],
    'upgrade-insecure-requests': true,
  }
}
