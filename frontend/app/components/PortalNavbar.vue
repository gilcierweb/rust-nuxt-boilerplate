<template>
  <header class="navbar border-b border-base-content/10 bg-base-100 px-4 lg:px-6">
    <div class="flex w-full items-center justify-between">
      <div class="flex items-center gap-3">
        <button
          type="button"
          class="btn btn-square btn-ghost lg:hidden"
          aria-label="Abrir menu"
          @click="sidebarOpen = true"
        >
          <span class="icon-[tabler--menu-2] size-5"></span>
        </button>
        <div>
          <p class="text-sm font-semibold text-base-content">{{ appName }}</p>
          <p class="text-xs text-base-content/60">Portal do Cliente</p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <div class="hidden text-right sm:block">
          <p class="text-sm font-medium text-base-content">{{ userEmail }}</p>
          <p class="text-xs text-base-content/60">{{ userRoleLabel }}</p>
        </div>
        <button type="button" class="btn btn-soft btn-sm" @click="logout">
          <span class="icon-[tabler--logout-2] size-4"></span>
          Sair
        </button>
      </div>
    </div>
  </header>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const authStore = useAuthStore()
const sidebarState = useState('portal-sidebar-open', () => false)

const appName = computed(() => runtimeConfig.public.appName || 'Contifya')
const userEmail = computed(() => authStore.user?.email || 'Cliente')
const userRoleLabel = computed(() =>
  authStore.isAdmin ? 'Administrador' : 'Cliente',
)

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
