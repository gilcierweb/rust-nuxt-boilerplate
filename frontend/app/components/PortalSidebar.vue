<template>
  <button
    v-if="sidebarOpen"
    type="button"
    class="fixed inset-0 z-40 bg-base-content/40 lg:hidden"
    aria-label="Fechar menu"
    @click="sidebarOpen = false"
  ></button>

  <aside
    class="fixed inset-y-0 left-0 z-50 w-72 -translate-x-full border-r border-base-content/10 bg-base-100 transition-transform lg:translate-x-0"
    :class="sidebarOpen ? 'translate-x-0' : ''"
  >
    <div class="flex h-full flex-col">
      <div class="flex items-center justify-between border-b border-base-content/10 px-4 py-4">
        <div>
          <p class="text-sm font-semibold text-base-content">{{ appName }}</p>
          <p class="text-xs text-base-content/60">Área do cliente</p>
        </div>
        <button
          type="button"
          class="btn btn-circle btn-ghost btn-sm lg:hidden"
          aria-label="Fechar"
          @click="sidebarOpen = false"
        >
          <span class="icon-[tabler--x] size-4"></span>
        </button>
      </div>

      <nav class="flex-1 overflow-y-auto p-3">
        <ul class="menu menu-sm gap-1">
          <li v-for="item in items" :key="item.to">
            <NuxtLink
              :to="item.to"
              class="inline-flex items-center gap-2"
              :class="isActive(item.to) ? 'menu-active' : ''"
              @click="sidebarOpen = false"
            >
              <span :class="`icon-[tabler--${item.icon}] size-4.5`"></span>
              <span>{{ item.label }}</span>
            </NuxtLink>
          </li>
        </ul>
      </nav>
    </div>
  </aside>
</template>

<script setup lang="ts">
const runtimeConfig = useRuntimeConfig()
const route = useRoute()
const sidebarState = useState('portal-sidebar-open', () => false)

const appName = computed(() => runtimeConfig.public.appName || 'Rust Nuxt Boilerplate')

const sidebarOpen = computed({
  get: () => sidebarState.value,
  set: (value: boolean) => {
    sidebarState.value = value
  },
})

const items = [
  { to: '/portal/dashboard', label: 'Dashboard', icon: 'layout-dashboard' },
  { to: '/portal/support', label: 'Suporte', icon: 'message-circle-exclamation' },
]

function isActive(path: string) {
  return route.path === path || route.path.startsWith(`${path}/`)
}
</script>