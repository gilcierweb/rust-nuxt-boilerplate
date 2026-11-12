import { isAuthPage, isPublicRoute } from "~/utils/auth-routes";
import { noticeTarget } from "~/utils/flash-notice";

function isAssetLikePath(path: string) {
  return /\.[a-zA-Z0-9]+$/.test(path);
}

/**
 * Detect if the route contains "undefined" as a path segment (e.g., /admin/audit-logs/undefined)
 * This catches malformed navigations before they reach components.
 */
function hasUndefinedSegment(path: string): string | null {
  const segments = path.split('/').filter(Boolean)
  const undefinedIndex = segments.findIndex((s) => s === 'undefined')
  if (undefinedIndex === -1) return null
  // Return the resource name (segment before "undefined") for proper redirect
  return segments[undefinedIndex - 1] ?? null
}

export default defineNuxtRouteMiddleware(async (to) => {
  const nuxtApp = useNuxtApp();
  // The type generation of this project does not resolve the @pinia/nuxt
  // augmentation, so the instance is read through a narrow cast.
  const authStore = useAuthStore((nuxtApp as unknown as { $pinia?: Parameters<typeof useAuthStore>[0] }).$pinia);
  const localePath = useLocalePath();
  const isPublic = isPublicRoute(to.path);
  const authPage = isAuthPage(to.path);
  const isAdminRoute = to.path.startsWith("/admin") || to.path.startsWith("/en/admin") || to.path.startsWith("/es/admin");

  // BLOCK: intercept any route containing "undefined" as a path segment
  // This prevents `/admin/resource/undefined` navigations that were causing
  // spurious SSR requests and prefetches.
  const undefinedResource = hasUndefinedSegment(to.path)
  if (undefinedResource) {
    const redirectMap: Record<string, string> = {
      'audit-logs': localePath('/admin/audit-logs'),
      'roles': localePath('/admin/roles'),
      'users': localePath('/admin/users'),
    }
    return navigateTo(redirectMap[undefinedResource] ?? localePath('/admin/dashboard'), { redirectCode: 302 })
  }

  // Ignore dev/static-like paths accidentally routed through Vue Router.
  if (
    to.matched.length === 0 ||
    isAssetLikePath(to.path) ||
    to.path.startsWith("/_nuxt/")
  ) {
    return;
  }

  // ... rest of existing middleware

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
    return navigateTo(localePath("/auth/login"));
  }

  if (isLoggedIn && authPage) {
    return navigateTo(isAdmin ? localePath("/admin/dashboard") : localePath("/portal"));
  }

  if (isLoggedIn && isAdminRoute && !isAdmin) {
    // The operator asked for a screen their authority does not open. The
    // redirect carries the notice so the portal can say so: a silent bounce
    // reads as a broken link. The target is the dashboard the portal index
    // forwards to, not the index itself, so the notice crosses one hop
    // instead of dying on the second one.
    return navigateTo(noticeTarget('forbidden', localePath('/portal/dashboard')))
  }
});
