/**
 * Map a frontend error (from $fetch / $api / Pinia action) to a safe,
 * localized user-facing message.
 *
 * # Why this exists
 *
 * Before this helper, auth pages rendered `err.statusMessage || err.message`
 * directly. When the network or the Nitro proxy failed, those fields carried
 * raw diagnostics from the runtime/browser, e.g.:
 *
 *   [POST] "https://api.internal.example.com/api/v1/auth/login": <no response>
 *   NetworkError when attempting to fetch resource.
 *
 * That leaked the internal backend URL (topology disclosure) to the end user
 * and surfaced untranslated, scary text instead of a friendly i18n message.
 *
 * # What this helper does
 *
 * - Inspects `statusCode`, `response.status`, and the backend `error.code`
 *   field to pick a locale key.
 * - Falls back to a generic "unexpected error" message — NEVER to the raw
 *   `message` / `statusMessage`, which may contain URLs or stack traces.
 * - Returns a translated string ready for `{{ errorMsg }}` rendering. Vue
 *   auto-escapes interpolation, so the output is XSS-safe by construction;
 *   there is no need for a manual HTML-entity escaper (the previous
 *   `sanitizeInput` only mangled apostrophes and slashes without removing
 *   any real threat).
 *
 * # Messages from the backend
 *
 * The backend returns RFC 7807-style error bodies for 4xx, with an optional
 * translated `message` field when `Accept-Language` is forwarded (see
 * `frontend/app/plugins/api.ts` `onRequest`). When the backend provides a
 * 4xx message we trust it — it is authored by us and already localized.
 * We only distrust 5xx and network-level messages, which can contain
 * infrastructural details.
 */

/** i18n function compatible with @nuxtjs/i18n `t` and vue-i18n `t`. */
type TranslateFn = (key: string, named?: Record<string, unknown>) => string

/** Coerce any thrown value to a loose error object with the fields we read. */
interface LooseError {
  statusCode?: number
  status?: number
  statusMessage?: string
  message?: string
  data?: {
    error?: { code?: string; message?: string }
  }
  response?: {
    status?: number
    statusText?: string
    _data?: {
      error?: { code?: string; message?: string }
    }
  }
}

function asLooseError(err: unknown): LooseError {
  if (err && typeof err === 'object') return err as LooseError
  return {}
}

/**
 * Map an auth-related error to a safe localized message.
 *
 * @param err        The thrown value from `$api` / `$fetch` / Pinia action.
 * @param t          The i18n translate function (typically `t` from useI18n).
 * @param fallbackKey Optional locale key used as the final fallback.
 *                     Defaults to 'common.errorExtended.unexpected'.
 */
export function mapAuthError(
  err: unknown,
  t: TranslateFn,
  fallbackKey = 'common.errorExtended.unexpected',
): string {
  const e = asLooseError(err)

  // Resolve HTTP status from any of the known locations across ofetch/H3.
  const status: number | undefined =
    e.statusCode ??
    e.response?.status ??
    e.status

  // Native browser fetch failures (NetworkError, CSP block, CORS, DNS) arrive
  // without a status code — the user must never see the raw message here.
  const isNetworkLevel = status === undefined

  // Backend error code ("INVALID_CREDENTIALS", "EMAIL_NOT_CONFIRMED", …).
  const code: string | undefined =
    e.data?.error?.code ?? e.response?._data?.error?.code

  // Backend-authored localized message. Trusted only for 4xx.
  const backendMessage: string | undefined =
    e.data?.error?.message ?? e.response?._data?.error?.message

  // 4xx — backend rejected the auth request with a known client error.
  if (status !== undefined && status >= 400 && status < 500) {
    // Prefer explicit codes that a login/register UX can present distinctly.
    switch (code) {
      case 'INVALID_CREDENTIALS':
        return t('auth.login.error.invalidCredentials')
      case 'EMAIL_NOT_CONFIRMED':
        return t('auth.login.error.emailNotConfirmed')
      case 'ACCOUNT_LOCKED':
        return t('auth.login.error.accountLocked')
      case 'EMAIL_ALREADY_EXISTS':
        return t('auth.register.error.emailExists')
      case 'WEAK_PASSWORD':
        return t('auth.register.error.weakPassword')
      case 'OTP_REQUIRED':
      case 'OTP_INVALID':
        return t('auth.login.otp.invalidCode')
      case 'TOO_MANY_REQUESTS':
      case 'RATE_LIMITED':
        return t('common.errorExtended.rateLimited')
      default:
        // Trust the backend message for 4xx — it is authored by us and
        // localized via Accept-Language. Fall back to a local key if missing.
        if (backendMessage && backendMessage.trim()) return backendMessage
        return t(fallbackKey)
    }
  }

  // 5xx / network-level — never leak server internals or URLs.
  if (isNetworkLevel || (status !== undefined && status >= 500)) {
    return t('common.errorExtended.network')
  }

  // Unknown shape — refuse to render any raw field.
  return t(fallbackKey)
}
