// Nitro plugin: validate runtime security posture on server startup.
//
// SECURITY_AUDIT.md I8: NUXT_PUBLIC_API_BASE should remain empty (default)
// in production. If set, it:
//   1. Bakes the backend URL into the client bundle (leaks deployment info).
//   2. Allows the browser to hit the backend directly, bypassing Nitro's
//      reverse proxy + cookie/session handling.
//
// This plugin emits warnings (NOT errors) at startup:
//   - In production: warns strongly; lists risks; emits remediation guidance.
//   - In non-production: warns lightly so devs aren't surprised.
// Plugin does NOT change behavior — only logs.
export default defineNitroPlugin(() => {
  const config = useRuntimeConfig();

  // Nitro / Node.js exposes the NODE_ENV that Nuxt sets during build/runtime.
  // We use it as a proxy for "production-like" environments.
  const env =
    (process.env.NUXT_ENV as string) ||
    (process.env.NODE_ENV as string) ||
    'development';
  const isProdLike = env === 'production' || env === 'staging';

  // -- 1. apiDirectBase: warn if set, especially in prod ------------------------
  const apiDirectBase = (config.public.apiDirectBase || '') as string;
  if (apiDirectBase) {
    // Common foot-guns we want to highlight
    const isHttp = /^http:\/\//i.test(apiDirectBase);
    const isLocalhost = /^(http:\/\/)?(localhost|127\.0\.0\.1|0\.0\.0\.0)/i.test(
      apiDirectBase,
    );
    const isProbablyExample = /example\.com|change-me|placeholder/i.test(apiDirectBase);

    if (isProdLike) {
      // Production: WARN strongly, do not panic — production might have a
      // legitimate reason (e.g. CDN proxying the backend).
      console.warn(
        '\n' +
          '╔══════════════════════════════════════════════════════════╗\n' +
          '║        SECURITY: NUXT_PUBLIC_API_BASE is SET in prod   ║\n' +
          '╠══════════════════════════════════════════════════════════╣\n' +
          '║ The backend URL is baked into the client bundle.        ║\n' +
          '║ Review SECURITY_AUDIT.md S14 / I8 before deployment.   ║\n' +
          '║ Recommended: leave empty to use the Nitro /api/v1 proxy.║\n' +
          '╚══════════════════════════════════════════════════════════╝\n' +
          `  current value: ${apiDirectBase}\n`,
      );
    } else if (isHttp || isLocalhost || isProbablyExample) {
      // Dev: warn lightly so it's surfaced during local onboarding.
      console.warn(
        `[security-check] NUXT_PUBLIC_API_BASE="${apiDirectBase}" → ` +
          (isHttp
            ? 'plain HTTP — use https:// in non-dev environments'
            : isLocalhost
              ? 'localhost — Nitro proxy is preferable for local dev too'
              : `looks like a placeholder (${apiDirectBase})`) +
          '\n',
      );
    }
  }

  // -- 2. Public runtime config: warn if any obviously-secret-shaped values
  const publicConfig = (config.public || {}) as Record<string, unknown>;
  const looksLikeSecret = (s: unknown): boolean => {
    if (typeof s !== 'string') return false;
    if (s.length < 16) return false;
    return /(sk_|pk_test|secret_?key|api[-_]?key)/i.test(s);
  };
  for (const [key, value] of Object.entries(publicConfig)) {
    if (key === 'apiDirectBase') continue; // Already handled above
    if (looksLikeSecret(value)) {
      console.warn(
        `[security-check] runtimeConfig.public.${key} looks like a secret value. ` +
          `public.* is shipped to the client — move to runtimeConfig.* (private) instead. ` +
          `value: ${String(value).slice(0, 6)}…\n`,
      );
    }
  }

  // -- 3. Stripe key check: should be the publishable key in production -------
  const stripeKey = (config.public.stripeKey || '') as string;
  if (stripeKey && isProdLike && stripeKey.startsWith('pk_test_')) {
    console.warn(
      '\n[security-check] ⚠️  NUXT_PUBLIC_STRIPE_KEY is pk_test_* in production environment.\n' +
        '         Stripe TEST keys (publishable pk_test_* / secret sk_test_*) are logged\n' +
        '         but should never be used for real payments. Switch to pk_live_*.\n',
    );
  }

  // -- 4. CDN URL sanity --------------------------------------------------------
  const cdnUrl = (config.public.cdnUrl || '') as string;
  if (cdnUrl && isProdLike && /^http:\/\//i.test(cdnUrl)) {
    console.warn(
      `[security-check] NUXT_PUBLIC_CDN_URL uses plain http:// in ${env}. ` +
        'Mixed content over HTTP may be blocked by browsers and reduces CSP integrity. ' +
        'Use https:// in production.\n',
    );
  }
});
