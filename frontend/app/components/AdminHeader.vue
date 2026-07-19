<template>
  <div class="bg-base-100 border-base-content/20 lg:ps-75 sticky top-0 z-50 flex border-b">
    <div class="mx-auto w-full max-w-7xl">
      <nav class="navbar py-2">
        <div class="navbar-start items-center gap-2">
          <button
            type="button"
            class="btn btn-soft btn-square btn-sm lg:hidden"
            aria-haspopup="dialog"
            aria-expanded="false"
            aria-controls="layout-sidebar"
            data-overlay="#layout-sidebar"
            @click="sidebarOpen = true"
          >
            <span class="icon-[tabler--menu-2] size-4.5"></span>
          </button>

          <!-- Search -->
          <div class="input no-focus border-0 px-0">
            <span class="icon-[tabler--search] text-base-content/80 my-auto me-2 size-4 shrink-0"></span>
            <input
              id="kbdInput"
              v-model="search"
              type="search"
              class="grow placeholder:text-sm"
              placeholder="Type to Search..."
            />
            <label class="sr-only" for="kbdInput">Search</label>
          </div>
        </div>

        <div class="navbar-end items-end gap-6">
          <!-- Profile Dropdown -->
          <div class="dropdown relative inline-flex [--offset:21]">
            <button
              id="profile-dropdown"
              type="button"
              class="dropdown-toggle avatar"
              aria-haspopup="menu"
              aria-expanded="false"
              aria-label="Dropdown"
            >
              <span class="bg-primary text-primary-content rounded-field flex size-9.5 items-center justify-center text-sm font-semibold">
                {{ initials }}
              </span>
            </button>
            <ul class="dropdown-menu dropdown-open:opacity-100 max-w-75 hidden w-full space-y-0.5" role="menu" aria-orientation="vertical" aria-labelledby="profile-dropdown">
              <li class="dropdown-header pt-4.5 mb-1 gap-4 px-5 pb-3.5">
                <div class="avatar avatar-online-top">
                  <div class="bg-primary text-primary-content flex w-10 items-center justify-center rounded-full text-sm font-semibold">
                    {{ initials }}
                  </div>
                </div>
                <div>
                  <h6 class="text-base-content mb-0.5 font-semibold">{{ displayName }}</h6>
                  <p class="text-base-content/80 font-medium">{{ rolesLabel }}</p>
                </div>
              </li>
              <li>
                <NuxtLink class="dropdown-item px-3" :to="localePath('/admin/dashboard')">
                  <span class="icon-[tabler--layout-dashboard] size-5"></span>
                  Dashboard
                </NuxtLink>
              </li>
              <li>
                <NuxtLink class="dropdown-item px-3" :to="localePath('/admin/users')">
                  <span class="icon-[tabler--users] size-5"></span>
                  Users
                </NuxtLink>
              </li>
              <li>
                <NuxtLink class="dropdown-item px-3" :to="localePath('/admin/roles')">
                  <span class="icon-[tabler--shield] size-5"></span>
                  Roles
                </NuxtLink>
              </li>
              <li>
                <NuxtLink class="dropdown-item px-3" :to="localePath('/admin/audit-logs')">
                  <span class="icon-[tabler--history] size-5"></span>
                  Audit Logs
                </NuxtLink>
              </li>
              <li>
                <hr class="border-base-content/20 -mx-2 my-1" />
              </li>
              <li>
                <a class="dropdown-item px-3" href="/swagger-ui/" target="_blank">
                  <span class="icon-[tabler--api] size-5"></span>
                  API documentation
                </a>
              </li>
              <li>
                <NuxtLink class="dropdown-item px-3" :to="localePath('/')">
                  <span class="icon-[tabler--home] size-5"></span>
                  Back to site
                </NuxtLink>
              </li>
              <li class="dropdown-footer p-2 pt-1">
                <button
                  type="button"
                  class="btn btn-text btn-error btn-block h-11 justify-start px-3 font-normal"
                  @click="handleLogout"
                >
                  <span class="icon-[tabler--logout] size-5"></span>
                  Logout
                </button>
              </li>
            </ul>
          </div>
        </div>
      </nav>
    </div>
  </div>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const authStore = useAuthStore()
const localePath = useLocalePath()
const search = useState('admin-shell-search', () => '')
const sidebarState = useState('admin-sidebar-open', () => false)

const appName = computed(() => runtimeConfig.public.appName || 'Telosync')
const userEmail = computed(() => authStore.user?.email || 'admin@telosync.local')
const rolesLabel = computed(() => authStore.user?.roles?.join(' / ') || 'admin')
const initials = computed(() => userEmail.value.slice(0, 2).toUpperCase())
const displayName = computed(() => `${appName.value} Admin`)

const sidebarOpen = computed({
  get: () => sidebarState.value,
  set: (value: boolean) => {
    sidebarState.value = value
  },
})

async function handleLogout() {
  await authStore.logout()
}
</script>
