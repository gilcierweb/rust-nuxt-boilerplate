<template>
  <aside
    id="layout-sidebar"
    class="overlay overlay-open:translate-x-0 drawer drawer-start sm:w-75 inset-y-0 start-0 hidden h-full [--auto-close:lg] lg:z-50 lg:block lg:translate-x-0 lg:shadow-none"
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
            <!-- Dashboard -->
            <li id="dashboard" class="accordion-item">
              <button
                class="accordion-toggle accordion-item-active:bg-neutral/10 inline-flex w-full items-center p-2 text-start text-sm font-normal"
                aria-controls="dashboard-collapse-dashboard"
                aria-expanded="true"
              >
                <span class="icon-[tabler--dashboard] size-4.5"></span>
                <span class="grow">{{ $t('admin.sidebar.dashboard') }}</span>
                <span class="icon-[tabler--chevron-right] accordion-item-active:rotate-90 size-4.5 shrink-0 transition-transform duration-300 rtl:rotate-180"></span>
              </button>
              <div
                id="dashboard-collapse-dashboard"
                class="accordion-content mt-1 hidden w-full overflow-hidden transition-[height] duration-300"
                aria-labelledby="dashboard"
                role="region"
              >
                <ul class="space-y-1">
                  <li>
                    <NuxtLink
                      :to="localePath('/admin/dashboard')"
                      class="inline-flex w-full items-center px-2"
                      :class="currentSlug === 'dashboard' ? 'menu-active' : ''"
                    >
                      <span>{{ $t('admin.sidebar.default') }}</span>
                    </NuxtLink>
                  </li>
                </ul>
              </div>
            </li>

            <!-- Section Divider -->
            <li class="text-base-content/50 before:bg-base-content/20 mt-2 p-2 text-xs uppercase before:absolute before:-start-3 before:top-1/2 before:h-0.5 before:w-2.5">
              {{ $t('admin.sidebar.management') }}
            </li>

            <!-- Management -->
            <li id="management" class="accordion-item">
              <button
                class="accordion-toggle accordion-item-active:bg-neutral/10 inline-flex w-full items-center p-2 text-start text-sm font-normal"
                aria-controls="management-collapse-management"
                aria-expanded="true"
              >
                <span class="icon-[tabler--settings] size-4.5"></span>
                <span class="grow">{{ $t('admin.sidebar.management') }}</span>
                <span class="icon-[tabler--chevron-right] accordion-item-active:rotate-90 size-4.5 shrink-0 transition-transform duration-300 rtl:rotate-180"></span>
              </button>
              <div
                id="management-collapse-management"
                class="accordion-content mt-1 hidden w-full overflow-hidden transition-[height] duration-300"
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

            <!-- Quick Links -->
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
import { onMounted, watch, nextTick, computed } from 'vue'
import { useRoute, useRuntimeConfig, useLocalePath } from '#imports'
import { useI18n } from 'vue-i18n'
import { useAuthStore } from '~/stores/auth'
import { ADMIN_RESOURCES, ADMIN_RESOURCE_SIDEBAR_LABELS } from '~/utils/admin-resources'

const { t } = useI18n()
const route = useRoute()
const runtimeConfig = useRuntimeConfig()
const authStore = useAuthStore()
const localePath = useLocalePath()

const appName = computed(() => runtimeConfig.public.appName || 'Rust Nuxt Boilerplate')
const userEmail = computed(() => authStore.user?.email || 'admin@example.com')
const initials = computed(() => userEmail.value.slice(0, 2).toUpperCase())

const currentSlug = computed(() => {
  const match = route.path.match(/\/admin\/([^/?#]+)/)
  return match?.[1] || 'dashboard'
})

const managementItems = computed(() =>
  ADMIN_RESOURCES
    .filter(r => r.group === 'management')
    .map(r => ({
      ...r,
      label: t(`admin.sidebar.${ADMIN_RESOURCE_SIDEBAR_LABELS[r.slug] || r.slug}`),
    })),
)

const ACCORDION_IDS = ['dashboard', 'management'] as const

function getOpenAccordionId(): string {
  if (currentSlug.value === 'dashboard') return 'dashboard'
  if (managementItems.value.some(m => m.slug === currentSlug.value)) return 'management'
  return 'dashboard'
}

function applyAccordionState(targetId: string) {
  const sidebar = document.querySelector('#layout-sidebar')
  if (!sidebar) return

  for (const id of ACCORDION_IDS) {
    const item = sidebar.querySelector<HTMLElement>(`#${id}`)
    if (!item) continue

    const content = item.querySelector<HTMLElement>('.accordion-content')
    const button = item.querySelector<HTMLElement>('.accordion-toggle')
    if (!content || !button) continue

    const shouldOpen = id === targetId

    if (shouldOpen) {
      item.classList.add('active')
      content.classList.remove('hidden')
      content.style.display = ''
      button.setAttribute('aria-expanded', 'true')
    } else {
      item.classList.remove('active')
      content.classList.add('hidden')
      content.style.display = ''
      button.setAttribute('aria-expanded', 'false')
    }
  }
}

function syncAccordion() {
  const sidebar = document.querySelector('#layout-sidebar')
  if (!sidebar) return

  const targetId = getOpenAccordionId()
  const items = sidebar.querySelectorAll<HTMLElement>('.accordion-item')

  items.forEach((item) => {
    const instance = (item as Record<string, unknown>) as { _hsAccordion?: { show(): void; hide(): void } }
    const shouldOpen = item.id === targetId

    if (instance._hsAccordion) {
      if (shouldOpen) instance._hsAccordion.show()
      else instance._hsAccordion.hide()
    } else {
      applyAccordionState(targetId)
    }
  })
}

onMounted(() => {
  nextTick(() => {
    setTimeout(() => {
      syncAccordion()
    }, 50)
  })
})

watch(
  () => route.path,
  () => {
    nextTick(() => {
      syncAccordion()
    })

    if (typeof window !== 'undefined' && window.HSOverlay) {
      const el = document.querySelector('#layout-sidebar')
      if (el) window.HSOverlay.close(el)
    }
  },
)
</script>
