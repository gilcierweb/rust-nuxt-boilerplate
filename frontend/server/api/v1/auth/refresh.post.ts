import {
  createError,
  getRequestURL,
  readRawBody,
} from 'h3'
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
    ['accept', 'content-type', 'cookie', 'user-agent'],
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
    let response: Awaited<ReturnType<typeof $fetch.raw<any>>> | null = null
    let lastError: any = null

    for (const backendApiBase of backendApiBases) {
      const targetUrls = [
        `${backendApiBase}/auth/refresh${requestUrl.search}`,
        `${backendApiBase}/auth/refresh/${requestUrl.search}`,
      ]

      for (const targetUrl of targetUrls) {
        try {
          response = await $fetch.raw(targetUrl, {
            method: 'POST',
            headers,
            body,
            redirect: 'manual',
            credentials: 'include',
          })
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

    return applyProxyResponse(event, response)
  } catch (error: any) {
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
