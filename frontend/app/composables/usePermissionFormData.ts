import { computed, type Ref } from 'vue'

import { useAdminRbac } from '~/composables/useAdminRbac'
import { mapAdminError } from '~/utils/admin-errors'
import { normalizeResourceResponse } from '~/utils/admin-resources'
import type { PermissionCatalog } from '~/types/rbac'

/**
 * The data a permission form needs before it can show anything: the catalog
 * that feeds the resource/action selects and the list of codes already in
 * use, so the form can warn about a collision before the submit. The two
 * requests travel in one `Promise.all`: the list is a probe for the
 * collision warning, the catalog feeds the selects.
 *
 * The edit page passes a getter as the key, so a navigation between two
 * rows refetches beside the item it displays (the key follows the route id).
 */
export async function usePermissionFormData(
  key: string | (() => string),
): Promise<{
  catalog: Ref<PermissionCatalog>
  codes: Ref<string[]>
  pending: Ref<boolean>
  error: Ref<string | null>
  refresh: () => Promise<void>
}> {
  const { t } = useI18n()
  const { data, pending, error, refresh } = await useApiAsyncData(key, async () => {
    const permissions = useAdminRbac().permissions
    const [catalogResponse, listResponse] = await Promise.all([
      permissions.catalog(),
      permissions.list({ per_page: 100 }),
    ])
    return {
      catalog: catalogResponse,
      codes: normalizeResourceResponse(listResponse).map((row: Record<string, unknown>) =>
        String(row.code ?? ''),
      ),
    }
  })

  return {
    catalog: computed(() => data.value?.catalog ?? { resources: [], actions: [] }),
    codes: computed(() => data.value?.codes ?? []),
    pending,
    error: computed<string | null>(() => (error.value ? mapAdminError(error.value, t) : '')),
    refresh,
  }
}
