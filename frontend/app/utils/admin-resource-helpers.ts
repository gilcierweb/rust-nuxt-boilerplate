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

  /**
   * Remove item with confirmation
   * @param item - Item to remove
   * @param config - Confirmation and API config
   */
  async function removeItem(
    item: Record<string, any>,
    config: {
      confirmMessage?: string
      successMessage?: string
      errorMessage?: string
      deleteEndpoint?: string
    } = {}
  ) {
    const {
      confirmMessage = `Tem certeza que deseja excluir este item?`,
      successMessage = 'Item excluído com sucesso',
      errorMessage = 'Erro ao excluir item',
      deleteEndpoint = `${basePath}/${item.id}`
    } = config

    if (!confirm(confirmMessage)) return

    try {
      await api.delete(deleteEndpoint)
      toast.success(successMessage)
      return true
    } catch (error: any) {
      toast.error(error?.message || errorMessage)
      return false
    }
  }

  return {
    navigateToShow,
    navigateToEdit,
    removeItem,
    showPath: (id: string) => localePath(useShowPath(resource, id)),
    editPath: (id: string) => localePath(useEditPath(resource, id)),
  }
}
