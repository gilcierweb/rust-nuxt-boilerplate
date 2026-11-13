import { mapAdminError } from '~/utils/admin-errors'

/**
 * Admin Resource Helpers
 * Centralized helper functions for admin CRUD operations
 * Follows Nuxt 4 composable patterns
 */

/**
 * Generate show page path for a resource
 * @param resource - Resource name (e.g., 'companies', 'branches')
 * @param id - Resource ID
 * @returns Full path to show page
 */
export function useShowPath(resource: string, id: string): string {
  return `/admin/${resource}/${id}`
}

/**
 * Generate edit page path for a resource
 * @param resource - Resource name (e.g., 'companies', 'branches')
 * @param id - Resource ID
 * @returns Full path to edit page
 */
export function useEditPath(resource: string, id: string): string {
  return `/admin/${resource}/${id}/edit`
}

/**
 * Composable for admin resource operations
 * @param resource - Resource name (e.g., 'companies', 'branches')
 * @param options - Optional configuration
 */
export function useAdminResource(resource: string, options?: {
  basePath?: string
}) {
  const api = useApi()
  const toast = useToast()
  const { t } = useI18n()
  const localePath = useLocalePath()
  const basePath = options?.basePath || `/admin/${resource}`

  /**
   * Navigate to show page
   */
  function navigateToShow(id: string) {
    return navigateTo(localePath(useShowPath(resource, id)))
  }

  /**
   * Navigate to edit page
   */
  function navigateToEdit(id: string) {
    return navigateTo(localePath(useEditPath(resource, id)))
  }

  // The confirmation is a dialog, not a native confirm: the native one blocks
  // the thread, cannot name the target and hides the consequence. The page
  // renders the dialog with these props and calls the handlers back.
  const removal = ref<{
    item: Record<string, any>
    title: string
    message: string
    confirmLabel: string
    cancelLabel: string
    successMessage: string
    errorKey: string
    deleteEndpoint: string
  } | null>(null)
  const removalPending = ref(false)

  /**
   * Open the confirmation dialog of a removal.
   */
  function requestRemoval(
    item: Record<string, any>,
    config: {
      title: string
      message: string
      confirmLabel?: string
      cancelLabel?: string
      successMessage: string
      errorKey: string
      deleteEndpoint?: string
    },
  ) {
    removal.value = {
      item,
      title: config.title,
      message: config.message,
      confirmLabel: config.confirmLabel ?? 'Excluir',
      cancelLabel: config.cancelLabel ?? 'Cancelar',
      successMessage: config.successMessage,
      errorKey: config.errorKey,
      deleteEndpoint: config.deleteEndpoint ?? `${basePath}/${item.id}`,
    }
  }

  function cancelRemoval() {
    removal.value = null
  }

  /**
   * Perform the confirmed removal and close the dialog.
   */
  async function confirmRemoval() {
    const target = removal.value
    if (!target) return false
    removalPending.value = true
    try {
      await api.delete(target.deleteEndpoint)
      toast.success(target.successMessage)
      removal.value = null
      return true
    } catch (error: any) {
      toast.error(mapAdminError(error, t, target.errorKey))
      return false
    } finally {
      removalPending.value = false
    }
  }

  return {
    navigateToShow,
    navigateToEdit,
    requestRemoval,
    confirmRemoval,
    cancelRemoval,
    removal,
    removalPending,
    showPath: (id: string) => localePath(useShowPath(resource, id)),
    editPath: (id: string) => localePath(useEditPath(resource, id)),
  }
}
