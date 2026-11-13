<template>
  <header class="navbar border-b border-base-content/10 bg-base-100 px-4 lg:px-6">
    <div class="flex w-full items-center justify-between">
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="btn btn-square btn-ghost lg:hidden"
          :aria-label="$t('portal.navbar.openMenu')"
          @click="sidebarOpen = true"
        >
          <span class="icon-[tabler--menu-2] size-5"></span>
        </button>
        <div>
          <p class="text-sm font-semibold text-base-content">{{ appName }}</p>
          <p class="text-xs text-base-content/60">{{ $t('portal.navbar.subtitle') }}</p>
        </div>
      </div>

      <div class="navbar-end items-center gap-2">
        <ThemeToggle />
        <LanguageSwitcher />

        <!-- The subject, its role and the way out travel together: a loose
             email, a loose role label and a permanent logout button spent the
             bar on three controls and squeezed it on a phone. The menu keeps
             the same information, named in the language of the viewer. -->
        <div class="dropdown relative inline-flex [--offset:21]">
          <button
            id="portal-profile-dropdown"
            type="button"
            class="dropdown-toggle avatar"
            aria-haspopup="menu"
            aria-expanded="false"
            :aria-label="$t('portal.navbar.userMenu')"
          >
            <span
              class="bg-primary text-primary-content rounded-field flex size-9.5 items-center justify-center text-sm font-semibold"
            >
              {{ initials }}
            </span>
          </button>
          <ul
            class="dropdown-menu dropdown-open:opacity-100 hidden w-full max-w-75 space-y-0.5"
            role="menu"
            aria-orientation="vertical"
            aria-labelledby="portal-profile-dropdown"
            tabindex="-1"
          >
            <li class="dropdown-header mb-1 gap-4 px-5 pt-4.5 pb-3.5">
              <div class="avatar avatar-online-top">
                <div
                  class="bg-primary text-primary-content flex w-10 items-center justify-center rounded-full text-sm font-semibold"
                >
                  {{ initials }}
                </div>
              </div>
              <div>
                <h6 class="text-base-content mb-0.5 font-semibold">{{ userEmail }}</h6>
                <p class="text-base-content/80 font-medium">{{ rolesLabel }}</p>
              </div>
            </li>
            <li>
              <NuxtLink class="dropdown-item px-3" :to="localePath('/portal/dashboard')">
                <span class="icon-[tabler--layout-dashboard] size-5"></span>
                {{ $t('portal.navbar.dashboard') }}
              </NuxtLink>
            </li>
            <li>
              <NuxtLink class="dropdown-item px-3" :to="localePath('/portal/support')">
                <span class="icon-[tabler--headset] size-5"></span>
                {{ $t('portal.navbar.support') }}
              </NuxtLink>
            </li>
            <li>
              <NuxtLink class="dropdown-item px-3" :to="localePath('/')">
                <span class="icon-[tabler--home] size-5"></span>
                {{ $t('portal.navbar.backToSite') }}
              </NuxtLink>
            </li>
            <li class="dropdown-footer p-2 pt-1">
              <button
                type="button"
                class="btn btn-text btn-error btn-block h-11 justify-start px-3 font-normal"
                @click="logout"
              >
                <span class="icon-[tabler--logout] size-5"></span>
                {{ $t('portal.navbar.logout') }}
              </button>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const authStore = useAuthStore()
const localePath = useLocalePath()
const { t, te } = useI18n()
const sidebarState = useState('portal-sidebar-open', () => false)

const appName = computed(() => runtimeConfig.public.appName || 'Rust Nuxt Boilerplate')
const userEmail = computed(() => authStore.user?.email || t('portal.navbar.fallbackName'))
const initials = computed(() => userEmail.value.slice(0, 2).toUpperCase())

/**
 * The presentation label of the subject roles: a known role of the catalog
 * names itself in the language of the viewer, an unknown one keeps its slug
 * rather than rendering a missing key or inventing a translation. The
 * existence check reads `te`, not a comparison of the answer: a translate
 * function answers the key itself when it is missing, and that answer is the
 * label of a role the locale simply does not know yet.
 */
const rolesLabel = computed(() => {
  const roles = authStore.user?.roles ?? []
  if (roles.length === 0) return ''
  return roles
    .map((role: string) => {
      const key = `portal.navbar.roles.${role}`
      return te(key) ? t(key) : role
    })
    .join(' / ')
})

const sidebarOpen = computed({
  get: () => sidebarState.value,
  set: (value: boolean) => {
    sidebarState.value = value
  },
})

async function logout() {
  await authStore.logout()
}
</script>
