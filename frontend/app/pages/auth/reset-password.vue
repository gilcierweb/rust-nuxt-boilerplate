<template>
  <div>
    <!-- Title -->
    <div class="mb-6">
      <h3 class="text-base-content text-2xl font-semibold mb-1">{{ $t('auth.resetPassword.title') }}</h3>
    </div>

    <div v-if="success" class="alert alert-success alert-soft text-center">
      <Icon name="heroicons:check-circle" class="h-10 w-10 mx-auto" />
      <div>
        <h3 class="font-semibold text-lg mb-1">{{ $t('auth.resetPassword.success.title') }}</h3>
        <NuxtLink :to="localePath('/auth/login')" class="btn btn-primary mt-4 inline-flex">{{ $t('auth.resetPassword.success.login') }}</NuxtLink>
      </div>
    </div>

    <div v-else-if="!token" class="alert alert-error alert-soft">
      <p class="text-sm">{{ $t('auth.resetPassword.error.invalidToken') }}</p>
      <NuxtLink :to="localePath('/auth/forgot-password')" class="btn btn-outline btn-primary mt-4 inline-flex text-sm">{{ $t('auth.resetPassword.error.requestNew') }}</NuxtLink>
    </div>

    <form v-else @submit.prevent="handleSubmit" class="space-y-4">
      <div v-if="errorMsg" class="alert alert-error alert-soft" role="alert">
        <Icon name="heroicons:exclamation-circle" class="h-5 w-5" />
        <p class="text-sm">{{ errorMsg }}</p>
      </div>

      <div>
        <label class="label-text mb-1.5 block" for="password">{{ $t('auth.resetPassword.newPassword') }}</label>
        <div class="input input-lg">
          <Icon name="heroicons:lock-closed" class="h-4 w-4 opacity-50" />
          <input 
            id="password"
            v-model="password" 
            :type="showPw ? 'text' : 'password'" 
            required 
            :placeholder="$t('auth.resetPassword.newPasswordPlaceholder')" 
          />
          <button 
            type="button" 
            class="text-base-content/50 hover:text-base-content transition-colors"
            @click="showPw = !showPw"
          >
            <Icon :name="showPw ? 'heroicons:eye-slash' : 'heroicons:eye'" class="h-4 w-4" />
          </button>
        </div>
      </div>

      <div>
        <label class="label-text mb-1.5 block" for="confirm">{{ $t('auth.resetPassword.confirmPassword') }}</label>
        <div class="input input-lg">
          <Icon name="heroicons:lock-closed" class="h-4 w-4 opacity-50" />
          <input 
            id="confirm"
            v-model="confirm" 
            :type="showPw ? 'text' : 'password'" 
            required 
            :placeholder="$t('auth.resetPassword.confirmPasswordPlaceholder')" 
          />
        </div>
      </div>

      <button type="submit" :disabled="loading || password !== confirm" class="btn btn-primary btn-lg btn-gradient btn-block">
        <Icon v-if="loading" name="svg-spinners:3-dots-fade" class="h-5 w-5" />
        <template v-else>{{ $t('auth.resetPassword.submit') }}</template>
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'auth' })
const { t } = useI18n()
const { $api } = useNuxtApp()
const localePath = useLocalePath()
const route = useRoute()
const token = route.query.token as string | undefined
const password = ref('')
const confirm = ref('')
const showPw = ref(false)
const loading = ref(false)
const success = ref(false)
const errorMsg = ref('')

async function handleSubmit() {
  if (password.value !== confirm.value) { errorMsg.value = t('auth.resetPassword.error.passwordMismatch'); return }
  loading.value = true; errorMsg.value = ''
  try {
    await $api('/auth/reset', { method: 'POST', body: { token, password: password.value, password_confirmation: confirm.value } })
    success.value = true
  } catch (err: any) {
    errorMsg.value = err.statusMessage || t('auth.resetPassword.error.invalidToken')
  } finally {
    loading.value = false }
}
</script>
