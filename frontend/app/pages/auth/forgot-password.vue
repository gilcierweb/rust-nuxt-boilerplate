<template>
  <div>
    <!-- Title -->
    <div class="mb-6">
      <h3 class="text-base-content text-2xl font-semibold mb-1">{{ $t('auth.forgotPassword.title') }}</h3>
      <p class="text-base-content/70 text-sm">
        {{ $t('auth.forgotPassword.remembered') }}
        <NuxtLink to="/auth/login" class="link link-primary font-medium">
          {{ $t('auth.forgotPassword.backToLogin') }}
        </NuxtLink>
      </p>
    </div>

    <!-- Success state -->
    <div
      v-if="sent"
      class="alert alert-success alert-soft mb-6 text-center"
    >
      <Icon name="heroicons:envelope-open" class="h-10 w-10 mx-auto" />
      <div>
        <h3 class="font-semibold text-lg mb-1">{{ $t('auth.forgotPassword.success.title') }}</h3>
        <p class="text-sm opacity-80">
          {{ $t('auth.forgotPassword.success.message') }}
        </p>
      </div>
    </div>

    <form v-else @submit.prevent="handleSubmit" class="space-y-4">
      <p class="text-base-content/70 text-sm leading-relaxed">
        {{ $t('auth.forgotPassword.description') }}
      </p>

      <div>
        <label class="label-text mb-1.5 block" for="email">{{ $t('auth.forgotPassword.email') }}</label>
        <div class="input input-lg">
          <Icon name="heroicons:envelope" class="h-4 w-4 opacity-50" />
          <input
            id="email"
            v-model="email"
            type="email"
            required
            autocomplete="email"
            :placeholder="$t('auth.forgotPassword.emailPlaceholder')"
            :disabled="loading"
          />
        </div>
      </div>

      <button type="submit" :disabled="loading" class="btn btn-primary btn-lg btn-gradient btn-block">
        <Icon v-if="loading" name="svg-spinners:3-dots-fade" class="h-5 w-5" />
        <template v-else>
          <Icon name="heroicons:paper-airplane" class="h-4 w-4" />
          {{ $t('auth.forgotPassword.submit') }}
        </template>
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })
const { $api } = useNuxtApp()
const email = ref('')
const loading = ref(false)
const sent = ref(false)

async function handleSubmit() {
  loading.value = true
  try {
    await $api('/auth/forgot-password', { method: 'POST', body: { email: email.value } })
  } catch {}
  sent.value = true
  loading.value = false
}
</script>
