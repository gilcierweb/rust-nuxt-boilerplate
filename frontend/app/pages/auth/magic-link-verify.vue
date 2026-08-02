<template>
  <div class="space-y-6">
    <div class="flex items-center gap-3">
      <NuxtLink :to="localePath('/')" class="flex items-center gap-3">
        <span class="text-primary">
          <svg width="32" height="32" viewBox="0 0 34 34" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="34" height="34" rx="8.5" fill="currentColor" fill-opacity="0.15" />
            <path d="M10 23L16.8 11H18.4L25 23H21.8L17.6 15.3L13.2 23H10Z" fill="currentColor" />
          </svg>
        </span>
        <h2 class="text-xl font-bold text-base-content">{{ $t('common.appName') }}</h2>
      </NuxtLink>
    </div>

    <div>
      <h3 class="mb-1.5 text-2xl font-semibold text-base-content">{{ $t('auth.magicLink.verifyTitle') }}</h3>
      <p class="text-base-content/80">{{ $t('auth.magicLink.verifyDescription') }}</p>
    </div>

    <Transition enter-active-class="duration-300 ease-out" enter-from-class="opacity-0 -translate-y-2">
      <div v-if="errorMsg" class="alert alert-error alert-soft text-sm">
        <span class="icon-[tabler--alert-circle] size-5"></span>
        <span>{{ errorMsg }}</span>
      </div>
    </Transition>

    <Transition enter-active-class="duration-300 ease-out" enter-from-class="opacity-0 -translate-y-2">
      <div v-if="successMsg" class="alert alert-success alert-soft text-sm">
        <span class="icon-[tabler--check-circle] size-5"></span>
        <span>{{ successMsg }}</span>
      </div>
    </Transition>

    <div v-if="isLoading" class="flex flex-col items-center gap-4">
      <span class="icon-[tabler--loader-2] size-10 animate-spin text-primary"></span>
      <p class="text-base-content/80">{{ $t('auth.magicLink.verifying') }}</p>
    </div>

    <div v-else class="space-y-4">
      <p class="text-center text-base-content/80">
        {{ $t('auth.magicLink.tryAgain') }}
        <NuxtLink :to="localePath('/auth/magic-link')" class="link link-animated link-primary font-normal">
          {{ $t('auth.magicLink.requestNew') }}
        </NuxtLink>
      </p>

      <p class="text-center text-base-content/80">
        {{ $t('auth.login.backToLogin') }}
        <NuxtLink :to="localePath('/auth/login')" class="link link-animated link-primary font-normal">
          {{ $t('auth.login.submit') }}
        </NuxtLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { mapAuthError } from '~/utils/auth-errors'

definePageMeta({
  layout: 'auth',
})

const { t } = useI18n()
const route = useRoute()
const router = useRouter()
const localePath = useLocalePath()

const isLoading = ref(true)
const errorMsg = ref('')

onMounted(async () => {
  const token = route.query.token as string | undefined
  if (!token) {
    errorMsg.value = t('auth.magicLink.invalid')
    isLoading.value = false
    return
  }

  try {
    const { $api } = useNuxtApp()
    await $api.post('/auth/magic-link/verify', { token })
    await router.push(localePath('/admin/dashboard'))
  } catch (err: any) {
    errorMsg.value = mapAuthError(err, t, 'auth.magicLink.invalid')
  } finally {
    isLoading.value = false
  }
})
</script>