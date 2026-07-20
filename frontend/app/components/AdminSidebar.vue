<template>
  <button
    v-if="sidebarOpen"
    type="button"
    class="overlay overlay-open:bg-opacity-0 fixed inset-0 z-40 bg-base-content/50 lg:hidden"
    :aria-label="$t('admin.sidebar.closeSidebar')"
    data-overlay="#layout-sidebar"
    @click="sidebarOpen = false"
  ></button>

  <aside
    id="layout-sidebar"
    class="overlay overlay-open:translate-x-0 drawer drawer-start sm:w-75 inset-y-0 start-0 hidden h-full [--auto-close:lg] lg:z-50 lg:block lg:translate-x-0 lg:shadow-none"
    :class="sidebarOpen ? '!block translate-x-0' : ''"
    :aria-label="$t('admin.sidebar.sidebar')"
    tabindex="-1"
  >
    <div class="drawer-body border-base-content/20 h-full border-e p-0">
      <div class="flex h-full max-h-full flex-col">
        <button
          type="button"
          class="btn btn-text btn-circle btn-sm absolute end-3 top-3 lg:hidden"
          :aria-label="$t('admin.sidebar.close')"
          data-overlay="#layout-sidebar"
          @click="sidebarOpen = false"
        >
          <span class="icon-[tabler--x] size-4.5"></span>
        </button>

        <div class="text-base-content border-base-content/20 flex flex-col items-center gap-4 border-b px-4 py-6">
          <div class="avatar">
            <div class="bg-primary text-primary-content flex size-17 items-center justify-center rounded-full text-2xl font-semibold">
              {{ initials }}
            </div>
          </div>
          <div class="text-center">
            <h3 class="text-base-content text-lg font-semibold">{{ appName }} {{ $t('admin.sidebar.admin') }}</h3>
            <p class="text-base-content/80">{{ userEmail }}</p>
          </div>
          <div class="flex gap-3">
            <a href="/swagger-ui/" target="_blank" class="link size-4.5" :aria-label="$t('admin.sidebar.apiDocs')">
              <span class="icon-[tabler--api] size-4.5"></span>
            </a>
          </div>
        </div>

        <div class="h-full overflow-y-auto">
          <ul class="accordion menu menu-sm gap-1 p-3">
            <li class="active accordion-item" id="dashboard">
              <button
                class="accordion-toggle accordion-item-active:bg-neutral/10 inline-flex w-full items-center p-2 text-start text-sm font-normal"
                :class="{ 'accordion-item-active:bg-neutral/10': isDashboardActive }"
                aria-controls="dashboard-collapse"
                :aria-expanded="dashboardOpen"
                @click="toggleAccordion('dashboard')"
              >
                <span class="icon-[tabler--dashboard] size-4.5"></span>
                <span class="grow">{{ $t('admin.sidebar.dashboard') }}</span>
                <span
                  class="icon-[tabler--chevron-right] size-4.5 shrink-0 transition-transform duration-300 rtl:rotate-180"
                  :class="{ 'rotate-90': dashboardOpen }"
                ></span>
              </button>
              <div
                id="dashboard-collapse"
                class="accordion-content mt-1 w-full overflow-hidden transition-[height] duration-300"
                :class="dashboardOpen ? 'block' : 'hidden'"
                aria-labelledby="dashboard"
                role="region"
              >
                <ul class="space-y-1">
                  <li>
                    <NuxtLink
                      :to="localePath('/admin/dashboard')"
                      class="inline-flex w-full items-center px-2"
                      :class="isDashboardActive ? 'menu-active' : ''"
                    >
                      <span>{{ $t('admin.sidebar.default') }}</span>
                    </NuxtLink>
                  </li>
                </ul>
              </div>
            </li>

            <li class="text-base-content/50 before:bg-base-content/20 mt-2 p-2 text-xs uppercase before:absolute before:-start-3 before:top-1/2 before:h-0.5 before:w-2.5">
              {{ $t('admin.sidebar.management') }}
            </li>
            <li class="accordion-item" id="management">
              <button
                class="accordion-toggle accordion-item-active:bg-neutral/10 inline-flex w-full items-center p-2 text-start text-sm font-normal"
                :class="{ 'accordion-item-active:bg-neutral/10': managementOpen }"
                aria-controls="management-collapse"
                :aria-expanded="managementOpen"
                @click="toggleAccordion('management')"
              >
                <span class="icon-[tabler--settings] size-4.5"></span>
                <span class="grow">{{ $t('admin.sidebar.management') }}</span>
                <span
                  class="icon-[tabler--chevron-right] size-4.5 shrink-0 transition-transform duration-300 rtl:rotate-180"
                  :class="{ 'rotate-90': managementOpen }"
                ></span>
              </button>
              <div
                id="management-collapse"
                class="accordion-content mt-1 w-full overflow-hidden transition-[height] duration-300"
                :class="managementOpen ? 'block' : 'hidden'"
                aria-labelledby="management"
                role="region"
              >
                <ul class="space-y-1">
                  <li v-for="item in managementItems" :key="item.slug">
                    <NuxtLink
                      :to="localePath(`/admin/${item.slug}`)"
                      class="inline-flex w-full items-center px-2"
                      :class="item.slug === currentSlug ? 'menu-active' : ''"
                    >
                      <span>{{ item.label }}</span>
                    </NuxtLink>
                  </li>
                </ul>
              </div>
            </li>

            <li class="text-base-content/50 before:bg-base-content/20 mt-2 p-2 text-xs uppercase before:absolute before:-start-3 before:top-1/2 before:h-0.5 before:w-2.5">
              {{ $t('admin.sidebar.quickLinks') }}
            </li>
            <li>
              <a href="/swagger-ui/" class="inline-flex w-full items-center px-2" target="_blank">
                <span class="icon-[tabler--api] size-4.5"></span>
                <span class="grow">{{ $t('admin.sidebar.apiDocs') }}</span>
              </a>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ADMIN_RESOURCES } from '~/utils/admin-resources'

const { t } = useI18n()
const route = useRoute()
const runtimeConfig = useRuntimeConfig()
const authStore = useAuthStore()
const localePath = useLocalePath()
const sidebarState = useState('admin-sidebar-open', () => false)

const appName = computed(() => runtimeConfig.public.appName || 'Rust Nuxt Boilerplate')
const userEmail = computed(() => authStore.user?.email || 'admin@example.com')
const initials = computed(() => userEmail.value.slice(0, 2).toUpperCase())

const sidebarOpen = computed({
  get: () => sidebarState.value,
  set: (value: boolean) => {
    sidebarState.value = value
  },
})

const basePath = computed(() => {
  const cleaned = route.path.replace(/^\/[a-z]{2}(-[A-Z]{2})?/, '')
  return cleaned || '/'
})

const currentSlug = computed(() => {
  const segments = basePath.value.replace('/admin/', '').split('/').filter(Boolean)
  return segments[0] || 'dashboard'
})

const isDashboardActive = computed(() => basePath.value === '/admin/dashboard')

const SLUG_TO_I18N: Record<string, string> = {
  'audit-logs': 'auditLogs',
}

const managementItems = computed(() => {
  return ADMIN_RESOURCES
    .filter((item) => item.group === 'management')
    .map((item) => ({
      ...item,
      label: t(`admin.sidebar.${SLUG_TO_I18N[item.slug] || item.slug}`),
    }))
})

const dashboardOpen = ref(isDashboardActive.value)
const managementOpen = ref(managementItems.value.some((item) => item.slug === currentSlug.value))

function toggleAccordion(id: string) {
  switch (id) {
    case 'dashboard':
      dashboardOpen.value = !dashboardOpen.value
      break
    case 'management':
      managementOpen.value = !managementOpen.value
      break
  }
}

watch(
  () => route.fullPath,
  () => {
    sidebarOpen.value = false
  },
)

watch(
  () => route.path,
  () => {
    dashboardOpen.value = isDashboardActive.value
    managementOpen.value = managementItems.value.some((item) => item.slug === currentSlug.value)
  },
  { immediate: true },
)
</script>