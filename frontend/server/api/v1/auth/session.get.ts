import {
  createError,
  getRequestURL,
} from 'h3'
import {
  applyProxyResponse,
  createForwardHeaders,
  requireBackendProxyConfig,
} from '~~/server/utils/proxy'

export default defineEventHandler(async (event) => {
  const { backendApiBases, backendApiKey } = requireBackendProxyConfig(event)

  const requestUrl = getRequestURL(event)
  const headers = createForwardHeaders(
    event,
    backendApiKey,
    ['accept', 'cookie', 'user-agent'],
    undefined,
    {
      filterCookies: true,
      allowedCookiePrefixes: [
        'refresh_token',
        'csrf',
      ],
    },
  )

  try {
    // The shape the proxy helper reads: the annotation of the raw response is
    // written out because the inferred type is circular through the catch
    // clause below.
    let response: { status: number; statusText: string; headers: Headers; _data: any } | null =
      null
    let lastError: any = null

    for (const backendApiBase of backendApiBases) {
      const targetUrl = `${backendApiBase}/auth/session${requestUrl.search}`
      try {
        response = (await $fetch.raw(targetUrl, {
          method: 'GET',
          headers,
          redirect: 'manual',
          credentials: 'include',
        })) as unknown as typeof response
        break
      } catch (error: any) {
        const status = error?.response?.status
        lastError = error
        if (status !== 404) {
          throw error
        }
        continue
      }
    }

    if (response) {
      return applyProxyResponse(
        event,
        response as unknown as Parameters<typeof applyProxyResponse>[1],
      )
    }

    throw lastError
} catch (error: any) {
      const response = error?.response
      if (!response) {
        throw createError({
          statusCode: 502,
          statusMessage: 'Backend API unavailable for auth session',
        })
      }

    return applyProxyResponse(event, response) ?? {
      error: response.statusText || 'Request failed',
    }
  }
})
