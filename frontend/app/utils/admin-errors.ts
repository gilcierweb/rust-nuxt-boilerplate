/**
 * Map a frontend error of an admin data path to a safe, localized message.
 *
 * The sibling `auth-errors.ts` serves the login and register pages; this one
 * serves the admin tables and forms, which read and write through the same
 * `$api` plugin and therefore see the same error shapes. Before it, the admin
 * paths rendered `extractErrorMessage`, which passed the backend message
 * through and answered a hardcoded English `'Could not load data.'` when the
 * body carried nothing: a pt-BR operator denied by the ability engine saw
 * the raw English `Admin domain permissions required`, and a body less
 * failure saw English fallback text.
 *
 * The policy, one for one with the auth helper:
 * - a denial (`FORBIDDEN`) and a throttled request name a locale key of
 *   their own, in the locale of the viewer: those sentences must read in
 *   the language of the screen, not in the language of the server;
 * - another 4xx keeps the backend message when it carries one. Be clear
 *   about what that message is: the refusals of the ability engine are
 *   built with `t!` against the boot locale, fixed to pt-BR in `app.rs`,
 *   so the text of an unmapped 4xx answers in pt-BR whatever the viewer.
 *   A key of ours wins wherever we have one, which is why the codes above
 *   are mapped instead of trusted;
 * - a 5xx or a network failure never leaks a URL or a runtime diagnostic;
 * - an unknown shape answers the fallback key, never a raw field.
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
 * Map an admin data error to a safe localized message.
 *
 * @param err        The thrown value from `$api` / `$fetch`.
 * @param t          The i18n translate function (typically `t` from useI18n).
 * @param fallbackKey Locale key of the final fallback, so a page may name a
 *                    message scoped to its own screen.
 */
export function mapAdminError(
  err: unknown,
  t: TranslateFn,
  fallbackKey = 'common.errorExtended.unexpected',
): string {
  const e = asLooseError(err)

  const status: number | undefined = e.statusCode ?? e.response?.status ?? e.status

  const code: string | undefined = e.data?.error?.code ?? e.response?._data?.error?.code

  const backendMessage: string | undefined = e.data?.error?.message ?? e.response?._data?.error?.message

  // A denial is the one code the admin screens must always name in their own
  // words: the operator did nothing wrong with the form, the policy refused
  // the write, and the sentence says so.
  if (code === 'FORBIDDEN') {
    return t('authorization.denied')
  }

  if (status !== undefined && status >= 400 && status < 500) {
    switch (code) {
      case 'TOO_MANY_REQUESTS':
      case 'RATE_LIMITED':
        return t('common.errorExtended.rateLimited')
      default:
        if (backendMessage && backendMessage.trim()) return backendMessage
        return t(fallbackKey)
    }
  }

  // 5xx and the statusless network failures answer the same sentence.
  if (status === undefined || status >= 500) {
    return t('common.errorExtended.network')
  }

  return t(fallbackKey)
}
