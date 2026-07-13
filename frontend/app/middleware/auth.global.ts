import { isAuthPage, isPublicRoute } from "~/utils/auth-routes";

function isAssetLikePath(path: string) {
  return /\.[a-zA-Z0-9]+$/.test(path);
}

export default defineNuxtRouteMiddleware(async (to) => {
  const nuxtApp = useNuxtApp();
  const authStore = useAuthStore(nuxtApp.$pinia);
  const event = useRequestEvent();
  const isPublic = isPublicRoute(to.path);
  const authPage = isAuthPage(to.path);
  const isHydrating =
    import.meta.client && nuxtApp.isHydrating && nuxtApp.payload.serverRendered;
  const cookieHeader = event?.node?.req?.headers?.cookie;
  const hasRefreshToken = cookieHeader?.includes("refresh_token") ?? false;

  // Ignore dev/static-like paths accidentally routed through Vue Router.
  if (
    to.matched.length === 0 ||
    isAssetLikePath(to.path) ||
    to.path.startsWith("/_nuxt/")
  ) {
    return;
  }

  if (import.meta.server) {
    if ((!isPublic || authPage) && hasRefreshToken && event) {
      try {
        // Extract requestFetch here where Nuxt context is guaranteed
        const requestFetch = useRequestFetch();
        const cookieHeader = event.node.req.headers.cookie;
        // Call store method with all required context
        await authStore.fetchSessionSSR(event, requestFetch, cookieHeader);
      } catch {
        authStore._clear();
        authStore.isInitialized = true;
      }
    } else if (!isPublic) {
      authStore._clear();
      authStore.isInitialized = true;
    }
  } else {
    // During hydration, only skip work if the SSR auth state is complete.
    if (isHydrating) {
      const hasCompleteHydratedSession =
        isPublic ||
        (authStore.isInitialized &&
          !!authStore.user &&
          (!!authStore.accessToken || !authStore.hasSession));

      if (hasCompleteHydratedSession) {
        return;
      }
    }

    if (!isPublic) {
      const needsBootstrap =
        !authStore.isInitialized ||
        (authStore.hasSession && !authStore.user) ||
        (authStore.hasSession && !authStore.accessToken);

      if (needsBootstrap) {
        try {
          await authStore.bootstrapSession();
        } catch (error: any) {
          // Bootstrap failed, likely session expired or unauthorized
          authStore._clear();
          authStore.isInitialized = true;
          const isUnauthorized =
            error?.statusCode === 401 ||
            error?.data?.error?.code === "UNAUTHORIZED";
          if (isUnauthorized) {
            authStore.setReturnUrl(to.fullPath);
            return navigateTo("/auth/login");
          }
        }
      }
    }
  }

  const isLoggedIn = authStore.isAuthenticated || authStore.hasActiveSession;
  const isAdmin = authStore.isAdmin;
  const isAdminRoute = to.path.startsWith("/admin");

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
