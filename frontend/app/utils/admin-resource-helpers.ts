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

  // The bulk removal reuses the dialog of the single one: the message names
  // the count, and the keys are translated with the real numbers of the run
  // once it ends, so a partial result reads as a partial result.
  const bulkRemoval = ref<{
    items: Record<string, any>[]
    title: string
    message: string
    confirmLabel: string
    cancelLabel: string
    successKey: string
    partialKey: string
    errorKey: string
    endpointFor: (item: Record<string, any>) => string
  } | null>(null)
  const bulkRemovalPending = ref(false)

  /**
   * Open the confirmation dialog of a bulk removal.
   */
  function requestBulkRemoval(
    items: Record<string, any>[],
    config: {
      title: string
      message: string
      confirmLabel?: string
      cancelLabel?: string
      successKey: string
      partialKey: string
      errorKey: string
      endpointFor?: (item: Record<string, any>) => string
    },
  ) {
    bulkRemoval.value = {
      items,
      title: config.title,
      message: config.message,
      confirmLabel: config.confirmLabel ?? t('admin.common.delete'),
      cancelLabel: config.cancelLabel ?? t('admin.common.dialog.cancel'),
      successKey: config.successKey,
      partialKey: config.partialKey,
      errorKey: config.errorKey,
      endpointFor: config.endpointFor ?? ((item) => `${basePath}/${item.id}`),
    }
  }

  function cancelBulkRemoval() {
    bulkRemoval.value = null
  }

  /**
   * Perform the confirmed bulk removal and close the dialog. The backend
   * answers one removal per request, so the bulk runs as a sequence of the
   * very same single removals: a failure keeps the remaining ids going, and
   * the report names how many of them went through.
   */
  async function confirmBulkRemoval(): Promise<{ deleted: number; failed: number }> {
    const target = bulkRemoval.value
    if (!target) return { deleted: 0, failed: 0 }
    bulkRemovalPending.value = true
    let deleted = 0
    let failed = 0
    let lastError: any = null
    try {
      for (const item of target.items) {
        try {
          await api.delete(target.endpointFor(item))
          deleted += 1
        } catch (error: any) {
          failed += 1
          lastError = error
        }
      }
      if (failed === 0) {
        toast.success(t(target.successKey, { count: deleted }))
      } else if (deleted > 0) {
        toast.error(
          t(target.partialKey, {
            deleted,
            total: target.items.length,
            failed,
          }),
        )
      } else {
        toast.error(mapAdminError(lastError, t, target.errorKey))
      }
      bulkRemoval.value = null
      return { deleted, failed }
    } finally {
      bulkRemovalPending.value = false
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
    requestBulkRemoval,
    confirmBulkRemoval,
    cancelBulkRemoval,
    bulkRemoval,
    bulkRemovalPending,
    showPath: (id: string) => localePath(useShowPath(resource, id)),
    editPath: (id: string) => localePath(useEditPath(resource, id)),
  }
}
