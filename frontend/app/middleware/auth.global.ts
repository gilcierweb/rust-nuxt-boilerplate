import { isAuthPage, isPublicRoute } from "~/utils/auth-routes";

function isAssetLikePath(path: string) {
  return /\.[a-zA-Z0-9]+$/.test(path);
}

export default defineNuxtRouteMiddleware(async (to) => {
  const nuxtApp = useNuxtApp();
  const authStore = useAuthStore(nuxtApp.$pinia);
  const isPublic = isPublicRoute(to.path);
  const authPage = isAuthPage(to.path);
  const isAdminRoute = to.path.startsWith("/admin");

  // Ignore dev/static-like paths accidentally routed through Vue Router.
  if (
    to.matched.length === 0 ||
    isAssetLikePath(to.path) ||
    to.path.startsWith("/_nuxt/")
  ) {
    return;
  }

  if (import.meta.server) {
    const event = useRequestEvent();
    const cookieHeader = event?.node?.req?.headers?.cookie;
    const hasRefreshToken = cookieHeader?.includes("refresh_token") ?? false;

    if ((!isPublic || authPage) && hasRefreshToken && event) {
      try {
        const requestFetch = useRequestFetch();
        const header = event.node.req.headers.cookie;
        await authStore.fetchSessionSSR(event, requestFetch, header);
      } catch (error: any) {
        // Only clear auth on authentication errors (401/403)
        // Network blips, rate limits, server errors should NOT log out the user
        const statusCode = error?.statusCode || error?.response?.status;
        if (statusCode === 401 || statusCode === 403) {
          authStore._clear();
        }
        authStore.isInitialized = true;
      }
    } else if (!isPublic) {
      authStore._clear();
      authStore.isInitialized = true;
    }
  } else {
    if (!isPublic) {
      const needsBootstrap =
        !authStore.isInitialized ||
        (authStore.hasSession && !authStore.user) ||
        (authStore.hasSession && !authStore.accessToken);

      if (needsBootstrap) {
        try {
          await authStore.bootstrapSession();
        } catch {
          authStore._clear();
          authStore.isInitialized = true;
        }
      }
    }
  }

  const isLoggedIn = authStore.isAuthenticated || authStore.hasActiveSession;
  const isAdmin = authStore.isAdmin;

  if (!isLoggedIn && !isPublic) {
    authStore.setReturnUrl(to.fullPath);
    return navigateTo("/auth/login");
  }

  if (isLoggedIn && authPage) {
    return navigateTo(isAdmin ? "/admin/dashboard" : "/portal");
  }

  if (isLoggedIn && isAdminRoute && !isAdmin) {
    return navigateTo("/portal");
  }
});
