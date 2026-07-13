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
  const configuredBaseURL = String(config.public.apiBase || "/api/v1");
  // On SSR always target Nuxt internal API routes to preserve proxy behavior.
  const baseURL = import.meta.server ? "/api/v1" : configuredBaseURL;

  function buildHeaders(inputHeaders?: HeadersInit) {
    const authStore = useAuthStore(nuxtApp.$pinia);
    const headers = new Headers(inputHeaders ?? {});
    const { csrf, headerName } = useCsrf();

    if (!headers.has("accept")) {
      headers.set("accept", "application/json");
    }

    if (import.meta.server) {
      const event = useRequestEvent();
      const requestHeaders = useRequestHeaders(["cookie", "user-agent"]);

      if (requestHeaders.cookie && !headers.has("cookie")) {
        headers.set("cookie", requestHeaders.cookie);
      }

      if (requestHeaders["user-agent"] && !headers.has("user-agent")) {
        headers.set("user-agent", requestHeaders["user-agent"]);
      }
    }

    let accessToken = authStore.accessToken;
    if (import.meta.server && !accessToken) {
      const event = useRequestEvent();
      accessToken = event?.context?.authAccessToken ?? null;
    }

    // nuxt-security/nuxt-csurf validates mutation requests on the Nuxt server.
    // For browser requests, always forward the CSRF token generated in the page meta.
    if (import.meta.client && csrf && headerName && !headers.has(headerName)) {
      headers.set(headerName, csrf);
    }

    return withAuthHeader(headers, accessToken);
  }

  const rawApi = $fetch.create({
    baseURL,
    credentials: "include",
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
      const canRefresh =
        import.meta.client &&
        !hasRetried &&
        isUnauthorized &&
        !isAuthRequest(request);

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
