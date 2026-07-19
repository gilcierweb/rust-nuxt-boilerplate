<template>
  <div class="text-center py-8">
    <div v-if="loading" class="space-y-4">
      <Icon name="svg-spinners:3-dots-fade" class="h-10 w-10 text-primary mx-auto" />
      <p class="text-base-content/60">{{ $t('auth.confirm.loading') }}</p>
    </div>

    <div v-else-if="confirmed" class="space-y-5">
      <div class="h-20 w-20 rounded-3xl bg-success/15 flex items-center justify-center mx-auto">
        <Icon name="heroicons:check-circle" class="h-10 w-10 text-success" />
      </div>
      <h1 class="text-2xl font-bold text-base-content font-display">{{ $t('auth.confirm.success.title') }}</h1>
      <p class="text-base-content/60 text-sm">{{ $t('auth.confirm.success.message') }}</p>
      <NuxtLink :to="localePath('/auth/login')" class="btn btn-primary btn-lg inline-flex">
        {{ $t('auth.confirm.success.login') }}
        <Icon name="heroicons:arrow-right" class="h-4 w-4" />
      </NuxtLink>
    </div>

    <div v-else class="space-y-5">
      <div class="h-20 w-20 rounded-3xl bg-error/15 flex items-center justify-center mx-auto">
        <Icon name="heroicons:x-circle" class="h-10 w-10 text-error" />
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
const token = route.query.token as string | undefined
const loading = ref(true)
const confirmed = ref(false)

onMounted(async () => {
  if (!token) { loading.value = false; return }
  try {
    await $api(`/auth/confirm?token=${token}`)
    confirmed.value = true
  } catch {}
  loading.value = false
})
</script>
