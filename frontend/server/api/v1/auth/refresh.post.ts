import {
  createError,
  getRequestURL,
  readRawBody,
} from 'h3'
import {
  fetchBackendWithRenderWakeupRetry,
  mapBackendWakeupExhaustion,
  type BackendRawResponse,
} from '~~/server/utils/backend-proxy-fetch'
import {
  applyProxyResponse,
  createForwardHeaders,
  requireBackendProxyConfig,
} from '~~/server/utils/proxy'

export default defineEventHandler(async (event) => {
  const { backendApiBases, backendApiKey } = requireBackendProxyConfig(
    event,
    true,
  )

  const requestUrl = getRequestURL(event)
  const headers = createForwardHeaders(
    event,
    backendApiKey,
    // 'csrf-token' is echoed by the SPA (client fetches) and by the SSR
    // refresh branch; the backend CSRF middleware requires it on the
    // cookie-authenticated refresh POST.
    ['accept', 'content-type', 'cookie', 'csrf-token', 'user-agent'],
    undefined,
    {
      filterCookies: true,
      allowedCookiePrefixes: [
        'refresh_token',
        'csrf',
      ],
    },
  )

  const body = await readRawBody(event, false)

  try {
    let response: BackendRawResponse<any> | null = null
    let lastError: any = null

    for (const backendApiBase of backendApiBases) {
      const targetUrls = [
        `${backendApiBase}/auth/refresh${requestUrl.search}`,
        `${backendApiBase}/auth/refresh/${requestUrl.search}`,
      ]

      for (const targetUrl of targetUrls) {
        try {
          response = await fetchBackendWithRenderWakeupRetry(
            targetUrl,
            backendApiBase,
            {
              method: 'POST',
              headers,
              body,
            },
          )
          break
        } catch (error: any) {
          const status = error?.response?.status
          lastError = error
          if (status !== 404) {
            throw error
          }
        }
      }

      if (response) {
        break
      }
    }

if (!response) {
      throw lastError
    }

    return applyProxyResponse(event, response as unknown as Parameters<typeof applyProxyResponse>[1])
  } catch (error: any) {
    const unavailableResponse = mapBackendWakeupExhaustion(
      event,
      error,
      { method: 'POST', path: 'auth/refresh' },
    )
    if (unavailableResponse) {
      return unavailableResponse
    }

    const response = error?.response
    if (!response) {
      throw createError({
        statusCode: 502,
        statusMessage: 'Backend API unavailable',
      })
    }

    return applyProxyResponse(event, response) ?? {
      error: response.statusText || 'Request failed',
    }
  }
})
