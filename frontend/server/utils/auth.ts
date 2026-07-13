import { getRequestHeaders, type H3Event } from 'h3'

function parseBearerToken(authorizationHeader?: string): string | null {
  if (!authorizationHeader) {
    return null
  }

  const value = authorizationHeader.trim()
  if (!value.toLowerCase().startsWith('bearer ')) {
    return null
  }

  const token = value.slice(7).trim()
  return token.length > 0 ? token : null
}

export async function resolveAccessTokenForProxy(
  event: H3Event,
): Promise<string | null> {
  const incomingHeaders = getRequestHeaders(event)
  const tokenFromAuthorization = parseBearerToken(incomingHeaders.authorization)
  if (tokenFromAuthorization) {
    return tokenFromAuthorization
  }

  if (!incomingHeaders.cookie?.includes('refresh_token')) {
    return null
  }

  try {
    const sessionResponse = await $fetch<{ access_token?: string }>(
      '/api/v1/auth/session',
      {
        headers: {
          cookie: incomingHeaders.cookie,
          accept: 'application/json',
        },
      },
    )
    return sessionResponse?.access_token || null
  } catch {
    return null
  }
}
