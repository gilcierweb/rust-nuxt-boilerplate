import { resolveRuntimeEnvVar } from '~~/server/utils/proxy'

/**
 * Cloudflare Cron Trigger keep-alive.
 *
 * Render free tier (and any hibernating host) sleeps after 15 min of
 * inactivity and takes 30-90s to boot. Pinging /health every 10 min keeps
 * the service warm so users never pay the cold start. This is the prevention
 * layer; the wake-up code in backend-proxy-fetch.ts is the recovery layer for
 * the rare case the service still sleeps (deploy, restart, cron miss).
 *
 * Enable by adding to wrangler.jsonc:
 *   "triggers": { "crons": ["*/10 * * * *"] }
 * The backend URL is read from NUXT_BACKEND_API_BASE / BACKEND_API_BASE at
 * runtime, exactly like the proxy.
 */
export default defineNitroPlugin((nitroApp) => {
  nitroApp.hooks.hook('cloudflare:scheduled', async ({ env, context }) => {
    // Make the binding visible to resolveRuntimeEnvVar (it reads globalThis.__env__).
    if (env && typeof globalThis !== 'undefined') {
      ;(globalThis as any).__env__ = env
    }

    const backendApiBase = resolveRuntimeEnvVar(
      { context: { _platform: { cloudflare: { env } } } } as any,
      'NUXT_BACKEND_API_BASE',
      'BACKEND_API_BASE',
    )

    if (!backendApiBase) {
      console.warn('[keep-alive] backend base not configured, skipping ping')
      return
    }

    const healthUrl = `${backendApiBase.replace(/\/+$/, '').replace(/\/api\/v\d+$/i, '')}/health`

    const ping = async () => {
      try {
        const response = await fetch(healthUrl, {
          method: 'GET',
          headers: { 'user-agent': 'keep-alive/1.0' },
        })
        console.info('[keep-alive] ping', healthUrl, response.status)
      } catch (error) {
        console.warn('[keep-alive] ping failed', healthUrl, String(error))
      }
    }

    context?.waitUntil?.(ping())
    await ping()
  })
})