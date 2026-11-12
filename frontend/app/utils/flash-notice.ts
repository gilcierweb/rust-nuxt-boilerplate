/**
 * A one-shot notice that survives a redirect, including an SSR one.
 *
 * The toast state of `useToast` lives in a client module: a toast pushed
 * while a route middleware answered on the server never hydrates, so the
 * redirect of a subject without the admin authority to the portal was
 * silent — the operator was simply moved somewhere else with no word
 * about why. A flash therefore travels with the navigation itself, as a
 * marker on the query of the target, and the page that lands reads the
 * marker, shows the toast and strips the marker from the address bar.
 *
 * The util holds no Vue or Nuxt import on purpose: the mapping from marker
 * to locale key is the contract worth testing, and both sides of it (the
 * middleware that writes the marker and the reader that spends it) call
 * these three functions instead of repeating the string.
 */

/** Every notice marker this application carries, with its locale key. */
export const NOTICE_KEYS = {
  /** The subject lacked the authority the route required. */
  forbidden: 'authorization.denied',
} as const

export type NoticeKind = keyof typeof NOTICE_KEYS

/** The target of a redirect that carries a notice. */
export function noticeTarget(kind: NoticeKind, path: string): string {
  return `${path}?notice=${kind}`
}

/**
 * The locale key of the notice a query carries, or null when it carries
 * none or one this version does not know.
 */
export function noticeKeyFromQuery(query: Record<string, unknown>): string | null {
  const notice = query.notice
  if (typeof notice !== 'string') return null
  if (!(notice in NOTICE_KEYS)) return null
  return NOTICE_KEYS[notice as NoticeKind]
}

/**
 * The query a forwarding redirect must carry to keep a notice alive.
 *
 * A redirect that builds its target from a path drops the query, so a
 * notice that already travelled one hop dies on the next one: the portal
 * index forwards to its dashboard, and the operator again landed with no
 * word about why. This answers the notice part of the query alone, or
 * undefined when there is none to keep, so a redirect forwards the flash
 * without forwarding the rest of the query it was given.
 */
export function noticeQuery(
  query: Record<string, unknown>,
): { notice: NoticeKind } | undefined {
  const notice = query.notice
  if (typeof notice !== 'string') return undefined
  if (!(notice in NOTICE_KEYS)) return undefined
  return { notice: notice as NoticeKind }
}
