import { computed } from 'vue'

/**
 * Read access to the class-level authority codes the subject holds, as
 * derived from the database grants: the backend's `MeResponse.permissions`
 * ("users:read", "roles:update", ...).
 *
 * The backend stays the enforcement; this is what lets the screens decide
 * which affordances to render. A subject with no permissions reads as
 * `[]`, so everything is `can(...) === false` until the session loads, which
 * is why write affordances also wait for `isAdmin` when they are admin-only.
 */
export function useRbac() {
  const authStore = useAuthStore()

  return {
    /** The class-level `resource:action` codes held at the subject. */
    permissions: computed<string[]>(() => authStore.user?.permissions ?? []),

    /** Whether the subject holds the class-level capability `code`. */
    can: (code: string): boolean =>
      authStore.user?.permissions?.includes(code) ?? false,
  }
}
