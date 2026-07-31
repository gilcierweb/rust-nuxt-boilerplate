function isAuthRequest(request: unknown) {
  const value =
    typeof request === "string"
      ? request
      : request instanceof Request
        ? request.url
        : String(request);

  return value.includes("/auth/");
}

function withAuthHeader(headers: Headers, accessToken: string | null) {
  if (accessToken && !headers.has("authorization")) {
    headers.set("authorization", `Bearer ${accessToken}`);
  }

  return headers;
}

export default defineNuxtPlugin((nuxtApp) => {
  const config = useRuntimeConfig();
  const publicConfig = config.public;
  
  /**
   * Decide which base URL to use based on execution context:
   * - Server-side (SSR): use relative `/api/v1` so the Nitro proxy routes the
   *   request through `server/api/v1/[...path].ts`. This is required because
   *   SSR fetches run inside the Nitro server process, and only the proxy can
   *   attach the correct internal cookies/headers.
   * - Client-side (CSR): use the absolute public API base URL
   *   (`NUXT_PUBLIC_API_BASE`), which bypasses the Nitro proxy round-trip for
   *   non-SSR requests. This reduces latency for client-initiated requests by
   *   going directly to the backend. The backend still receives the same
   *   cookies via `credentials: "include"` and any auth headers set below.
   */
  const baseURL = import.meta.server
    ? "/api/v1"
    : String(publicConfig.apiBase || "/api/v1").replace(/\/+$/, "");

  function buildHeaders(inputHeaders?: HeadersInit) {
    const authStore = useAuthStore(nuxtApp.$pinia);
    const headers = new Headers(inputHeaders ?? {});

    if (!headers.has("accept")) {
      headers.set("accept", "application/json");
    }

    if (import.meta.server) {
      const event = useRequestEvent();
      const requestHeaders = useRequestHeaders([
        "cookie",
        "user-agent",
        "accept-language",
      ]);

      if (requestHeaders.cookie && !headers.has("cookie")) {
        headers.set("cookie", requestHeaders.cookie);
      }

      if (requestHeaders["user-agent"] && !headers.has("user-agent")) {
        headers.set("user-agent", requestHeaders["user-agent"]);
      }

      // Forward the browser's Accept-Language so the backend can pick the
      // right locale for error messages. The browser sends this on every
      // navigation request, so it reflects the user's current preference.
      if (
        requestHeaders["accept-language"] &&
        !headers.has("accept-language")
      ) {
        headers.set("accept-language", requestHeaders["accept-language"]);
      }
    }

    // On the client, set Accept-Language from the active @nuxtjs/i18n locale
    // so the backend translates error messages in the language the user is
    // currently browsing. This always runs (even when the browser already
    // sent an Accept-Language header) so the backend always receives the
    // user's UI locale regardless of browser language preferences.
    //
    // `useI18n()` is a Vue composable that throws "Must be called at the top
    // of a setup function" when invoked from non-setup contexts (e.g., a
    // plugin's `onRequest` hook). We read the locale via `nuxtApp.$i18n`
    // instead — the underlying Vue I18n instance is available on every
    // Nuxt app and works outside of setup contexts.
    if (import.meta.client) {
      try {
        const $i18n = nuxtApp.$i18n as
          | {
              locale: { value: unknown };
              locales: { value: Array<{ code: string }> | unknown };
            }
          | undefined;
        const current = ($i18n?.locale.value as string | undefined) ?? "";
        const allLocales: Array<{ code: string }> = Array.isArray(
          $i18n?.locales.value,
        )
          ? ($i18n!.locales.value as Array<{ code: string }>)
          : [];
        const others = allLocales
          .map((l) => l.code)
          .filter((code) => code !== current);
        if (current) {
          const tail = others.map((code) => `${code};q=0.5`).join(", ");
          const finalHeader = tail
            ? `${current};q=1.0, ${tail}`
            : `${current};q=1.0`;
          headers.set("accept-language", finalHeader);
        }
      } catch {
        // $i18n unavailable — leave Accept-Language to whatever was set.
      }
    }

    let accessToken = authStore.accessToken;
    if (import.meta.server && !accessToken) {
      const event = useRequestEvent();
      accessToken = event?.context?.authAccessToken ?? null;
    }

    // Read CSRF token from cookie and send in header
    // Client-side: read from document.cookie
    // Server-side: parse from the forwarded cookie header
    const csrfToken = (() => {
      const cookieHeader = import.meta.client
        ? document.cookie
        : (headers.get("cookie") ?? "");
      const match = cookieHeader
        .split('; ')
        .find((row) => row.startsWith('csrf_token='));
      return match ? match.split('=')[1] : null;
    })();

    if (csrfToken && !headers.has("csrf-token")) {
      headers.set("csrf-token", csrfToken);
    }

    return withAuthHeader(headers, accessToken);
  }

  const rawApi = $fetch.create({
    baseURL,
    credentials: "include", // Important: send cookies
    retry: 0,
    onRequest({ options }) {
      options.headers = buildHeaders(options.headers);
    },
  });

  async function api<T>(
    request: string,
    options: Record<string, any> = {},
    hasRetried = false,
  ): Promise<T> {
    try {
      if (import.meta.server && request.startsWith("/")) {
        const requestFetch = useRequestFetch();
        const target = request.startsWith(baseURL)
          ? request
          : `${baseURL}${request}`;
        return await requestFetch<T>(target, {
          ...options,
          headers: buildHeaders(options.headers),
        });
      }

      return await rawApi<T>(request, options);
    } catch (error: any) {
      const authStore = useAuthStore(nuxtApp.$pinia);
      const isUnauthorized =
        error?.response?.status === 401 || error?.statusCode === 401;

      // Only retry refresh for TOKEN_EXPIRED — skip for TOKEN_REVOKED to
      // avoid wasted round-trips when the user has been logged out server-side.
      const errorCode = error?.data?.error?.code as string | undefined;
      const isTokenRevoked = errorCode === "TOKEN_REVOKED";

      const canRefresh =
        import.meta.client &&
        !hasRetried &&
        isUnauthorized &&
        !isAuthRequest(request) &&
        !isTokenRevoked;

      if (!canRefresh) {
        throw error;
      }

      try {
        await authStore.refreshTokens();
      } catch {
        authStore.$patch({
          accessToken: null,
          user: null,
          hasSession: false,
          isBootstrapping: false,
        });

        throw createError({
          statusCode: 401,
          statusMessage: "Unauthorized",
        });
      }

      return await api<T>(request, options, true);
    }
  }

  nuxtApp.provide("api", api);
});
