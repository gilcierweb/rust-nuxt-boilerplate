import {
  createError,
  getRequestURL,
  readRawBody,
} from 'h3'
import { resolveAccessTokenForProxy } from '~~/server/utils/auth'
import {
  applyProxyResponse,
  createForwardHeaders,
  requireBackendProxyConfig,
} from '~~/server/utils/proxy'

export default defineEventHandler(async (event) => {
  const { backendApiBases, backendApiKey } = requireBackendProxyConfig(event)
  const backendApiBase = backendApiBases[0]
  const path = event.context.params?.path || ''
  const requestUrl = getRequestURL(event)
  const targetUrl = `${backendApiBase}/${path}${requestUrl.search}`
  const accessToken = await resolveAccessTokenForProxy(event)

  const headers = createForwardHeaders(
    event,
    backendApiKey,
    [
      'authorization',
      'accept',
      'accept-language',
      'content-type',
      'cookie',
      'user-agent',
      'csrf-token',
    ],
    accessToken,
    {
      filterCookies: true,
      allowedCookiePrefixes: [
        'refresh_token',
        'csrf',
      ],
    },
  )

  const method = event.method || 'GET'
  const body = ['GET', 'HEAD'].includes(method)
    ? undefined
    : await readRawBody(event, false)

  try {
    const response = await $fetch.raw(targetUrl, {
      method,
      headers,
      body,
      redirect: 'manual',
      credentials: 'include',
    })

    return applyProxyResponse(event, response)
  } catch (error: any) {
    const response = error?.response
    if (!response) {
      // Unexpected error shape (no response) — log for debugging and rethrow.
      // The cause could be a network error (ECONNREFUSED, ENOTFOUND, abort),
      // a JSON parse error, or an unhandled throw inside ofetch. We log the
      // error name, message and any hint fields so the operator can diagnose
      // quickly without enabling verbose logging.
      const errorName = error?.name ?? 'UnknownError'
      const errorMessage = error?.message ?? String(error)
      const errorCode = error?.code ?? error?.cause?.code ?? null
      console.error(
        '[proxy] request error without response:',
        JSON.stringify({
          name: errorName,
          message: errorMessage,
          code: errorCode,
          method,
          path,
        }),
      )
      throw createError({
        statusCode: 502,
        statusMessage: 'Backend API unavailable',
        data: {
          reason: errorName,
          detail: errorMessage,
          code: errorCode,
        },
      })
    }

    const errorData = applyProxyResponse(event, response)

    // Ensure error response has proper structure with error.message
    if (errorData && typeof errorData === 'object' && (errorData as any).error) {
      return errorData
    }

    // Fallback to structured error response
    return {
      error: {
        code: 'ERROR',
        message: response.statusText || 'Request failed',
      },
    }
  }
})
