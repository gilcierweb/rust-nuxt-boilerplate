<template>
  <div class="text-center py-8">
    <div v-if="loading" class="space-y-4">
      <span class="icon-[tabler--loader-2] h-10 w-10 text-primary mx-auto animate-spin"></span>
      <p class="text-base-content/60">{{ $t('auth.confirm.loading') }}</p>
    </div>

    <div v-else-if="confirmed" class="space-y-5">
      <div class="h-20 w-20 rounded-3xl bg-success/15 flex items-center justify-center mx-auto">
        <span class="icon-[tabler--circle-check] h-10 w-10 text-success"></span>
      </div>
      <h1 class="text-2xl font-bold text-base-content font-display">{{ $t('auth.confirm.success.title') }}</h1>
      <p class="text-base-content/60 text-sm">{{ $t('auth.confirm.success.message') }}</p>
      <NuxtLink :to="localePath('/auth/login')" class="btn btn-primary btn-lg inline-flex">
        {{ $t('auth.confirm.success.login') }}
        <span class="icon-[tabler--arrow-right] h-4 w-4"></span>
      </NuxtLink>
    </div>

    <div v-else class="space-y-5">
      <div class="h-20 w-20 rounded-3xl bg-error/15 flex items-center justify-center mx-auto">
        <span class="icon-[tabler--circle-x] h-10 w-10 text-error"></span>
      </div>
      <h1 class="text-2xl font-bold text-base-content font-display">{{ $t('auth.confirm.error.title') }}</h1>
      <p class="text-base-content/60 text-sm">{{ $t('auth.confirm.error.message') }}</p>
      <NuxtLink :to="localePath('/auth/login')" class="btn btn-outline btn-primary inline-flex">
        {{ $t('auth.confirm.error.backToLogin') }}
      </NuxtLink>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })
const { $api } = useNuxtApp()
const localePath = useLocalePath()
const route = useRoute()
const router = useRouter()
const { token, consume: consumeToken } = useAuthTokenFromQuery(
  'token',
  route,
  router,
)
const loading = ref(true)
const confirmed = ref(false)

onMounted(async () => {
  if (!token) { loading.value = false; return }
  // The confirmation token is only ever transported in the URL, so strip it
  // from the address bar before calling the API.
  consumeToken()
  try {
    await $api(`/auth/confirm?token=${token}`)
    confirmed.value = true
  } catch {
    // The error state below already explains the failure generically, so the
    // catch only needs to leave confirmed=false for the template to render it.
  }
  loading.value = false
})
</script>
