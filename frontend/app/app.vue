<template>
  <div>
    <NuxtLoadingIndicator color="var(--color-primary)" />
    <NuxtRouteAnnouncer />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
    <AppToasts />
  </div>
</template>

<script setup lang="ts">
import AppToasts from '~/components/ui/AppToasts.vue'
import { noticeKeyFromQuery } from '~/utils/flash-notice'

const route = useRoute()
const router = useRouter()
const { t } = useI18n()
const toast = useToast()

// Spend a notice that a redirect carried (see utils/flash-notice): the toast
// state is client side, so a notice pushed while a middleware answered on
// the server never hydrates. The marker on the query travels with the
// navigation instead, and spending it here covers both the SPA navigation
// and the first client render after an SSR redirect, because the query is
// still on the address bar at hydration. The spending itself is client only:
// on the server there is no router to strip with and no toast to show.
if (import.meta.client) {
  watch(
    () => route.query.notice,
    () => {
      const key = noticeKeyFromQuery(route.query)
      if (!key) return
      toast.warning(t(key))
      // Strip only the marker, so a refresh or a back navigation does not
      // repeat it and the rest of the query survives.
      const { notice: _notice, ...rest } = route.query
      router.replace({ path: route.path, query: rest })
    },
    { immediate: true },
  )
}
</script>
