/**
 * The API base URL each execution context must use.
 *
 * The server always renders through the Nitro `/api/v1` proxy: the proxy
 * forwards the cookies of the browser request and attaches the access token it
 * resolves from the refresh cookie, so a server render is authenticated. A
 * direct call from the SSR process drops both, whatever base is configured.
 *
 * The client may call the backend directly when `NUXT_PUBLIC_API_BASE` is
 * configured, which skips the proxy round-trip for client-initiated requests.
 */
export function resolveApiBase(
  config: { public?: { apiBase?: string; apiDirectBase?: string } },
  isServer: boolean,
): string {
  if (isServer) {
    return '/api/v1'
  }

  const directBase = config.public?.apiDirectBase?.replace(/\/+$/, '')
  if (directBase) {
    return directBase
  }

  return config.public?.apiBase || '/api/v1'
}