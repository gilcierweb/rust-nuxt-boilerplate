import {
  AUTH_TOKEN_QUERY_PARAM,
  authTokenFromQuery,
  pathWithoutAuthToken,
} from '~~/shared/utils/auth-token-url'

/**
 * Read a one-time token from the current route, then remove it from the URL.
 *
 * Call `consume()` as soon as the token has been captured (before any async
 * work): the address bar must not keep showing a credential, whatever happens
 * to the request afterwards.
 *
 * The route and router default to Nuxt's auto-imports and are parameters so the
 * behaviour can be exercised without the Nuxt runtime.
 */
export function useAuthTokenFromQuery(
  param: string = AUTH_TOKEN_QUERY_PARAM,
  route: { path: string; query: Record<string, unknown> } = useRoute(),
  router: { replace: (location: { path: string }) => unknown } = useRouter(),
) {
  const token = authTokenFromQuery(route.query, param)

  function consume() {
    // Nothing to remove: replacing the URL here would only add a history entry
    // pointing at the same document.
    if (token === null) {
      return
    }

    router.replace({ path: pathWithoutAuthToken(route.path, route.query, param) })
  }

  return { token, consume }
}
