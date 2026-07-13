<template>
  <div class="space-y-6">
    <div class="flex items-center gap-3">
      <NuxtLink to="/" class="flex items-center gap-3">
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
      <h3 class="mb-1.5 text-2xl font-semibold text-base-content">{{ $t('auth.login.title') }}</h3>
      <p class="text-base-content/80">{{ $t('auth.login.noAccount') }}
        <NuxtLink to="/auth/register" class="link link-animated link-primary font-normal">
          {{ $t('auth.login.createAccount') }}
        </NuxtLink>
      </p>
    </div>

    <Transition enter-active-class="duration-300 ease-out" enter-from-class="opacity-0 -translate-y-2">
      <div v-if="errorMsg" class="alert alert-error alert-soft text-sm">
        <span class="icon-[tabler--alert-circle] size-5"></span>
        <span>{{ errorMsg }}</span>
      </div>
    </Transition>

    <div v-if="requiresOtp" class="space-y-4">
      <p class="text-sm text-base-content/80">{{ $t('auth.login.otp.label') }}</p>
      <div>
        <label class="label-text" for="otpCode">{{ $t('auth.login.otp.title') }}</label>
        <input
          id="otpCode"
          v-model="form.otp_code"
          type="text"
          inputmode="numeric"
          maxlength="6"
          :placeholder="$t('auth.login.otp.placeholder')"
          class="input text-center tracking-[0.5em]"
          autocomplete="one-time-code"
          @input="form.otp_code = ($event.target as HTMLInputElement)?.value?.replace(/\D/g, '') ?? ''"
        />
      </div>
      <button
        type="button"
        :disabled="isLoading || form.otp_code.length !== 6"
        class="btn btn-lg btn-primary btn-gradient btn-block"
        @click="handleOtpVerify"
      >
        <span v-if="isLoading" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
        <template v-else>{{ $t('auth.login.otp.title') }}</template>
      </button>
    </div>

    <form v-else class="space-y-4" @submit.prevent="handleLogin">
      <div class="flex items-center gap-3">
        <span class="text-base-content/80 text-sm">{{ $t('auth.login.loginWith') }}</span>
        <button type="button" class="link link-animated link-primary font-normal">
          {{ $t('auth.login.magicLink') }}
        </button>
      </div>

      <div class="flex flex-wrap gap-4 sm:gap-6">
        <button type="button" class="btn btn-outline btn-primary grow">{{ $t('auth.login.loginAsUser') }}</button>
        <button type="button" class="btn btn-outline btn-primary grow">{{ $t('auth.login.loginAsAdmin') }}</button>
      </div>

      <div>
        <label class="label-text" for="email">{{ $t('auth.login.email') }}*</label>
        <input
          id="email"
          v-model="form.email"
          type="email"
          required
          autocomplete="email"
          :placeholder="$t('auth.login.emailPlaceholder')"
          :disabled="isLoading"
          class="input"
        />
      </div>

      <div>
        <div class="mb-1.5 flex items-center justify-between">
          <label class="label-text" for="password">{{ $t('auth.login.password') }}*</label>
          <NuxtLink to="/auth/forgot-password" class="link link-animated link-primary font-normal">
            {{ $t('auth.login.forgotPassword') }}
          </NuxtLink>
        </div>
        <div class="input">
          <input
            id="password"
            v-model="form.password"
            :type="showPassword ? 'text' : 'password'"
            required
            autocomplete="current-password"
            :placeholder="$t('auth.login.passwordPlaceholder')"
            :disabled="isLoading"
          />
          <button type="button" class="block cursor-pointer" aria-label="toggle password visibility" @click="showPassword = !showPassword">
            <span :class="[showPassword ? 'hidden' : 'block', 'icon-[tabler--eye] size-5 shrink-0']" />
            <span :class="[showPassword ? 'block' : 'hidden', 'icon-[tabler--eye-off] size-5 shrink-0']" />
          </button>
        </div>
      </div>

      <div class="flex items-center justify-between gap-y-2">
        <label class="flex items-center gap-2">
          <input type="checkbox" class="checkbox checkbox-primary checkbox-sm" />
          <span class="label-text p-0 text-base text-base-content/80">{{ $t('auth.login.rememberMe') }}</span>
        </label>
      </div>

      <button type="submit" :disabled="isLoading" class="btn btn-lg btn-primary btn-gradient btn-block">
        <span v-if="isLoading" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
        <template v-else>{{ $t('auth.login.submit') }}</template>
      </button>
    </form>

    <p class="text-center text-base-content/80">
      {{ $t('auth.login.noAccount') }}
      <NuxtLink to="/auth/register" class="link link-animated link-primary font-normal">
        {{ $t('auth.login.createAccount') }}
      </NuxtLink>
    </p>

    <div class="divider">{{ $t('common.or') }}</div>
    <button type="button" class="btn btn-text btn-block" disabled>
      <span class="icon-[tabler--brand-google] size-5"></span>
      {{ $t('auth.login.submitWithGoogle') }}
    </button>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'auth',
})

const { t } = useI18n()
const authStore = useAuthStore()
const toast = useToast()

const form = reactive({
  email: '',
  password: '',
  otp_code: '',
})

const isLoading = ref(false)
const showPassword = ref(false)
const errorMsg = ref('')
const requiresOtp = ref(false)

/** Sanitize user input to prevent XSS - escapes HTML entities */
function sanitizeInput(input: string): string {
  if (!input) return ''
  return input
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#x27;')
    .replace(/\//g, '&#x2F;')
}

async function handleLogin() {
  errorMsg.value = ''
  isLoading.value = true

  try {
    const result = await authStore.login({
      email: form.email,
      password: form.password,
    })

    // Backend signals 2FA required
    if ((result as any)?.requires_otp) {
      requiresOtp.value = true
      return
    }

    toast.success(t('common.success'))
    const returnUrl = authStore.returnUrl || '/admin/dashboard'
    authStore.returnUrl = null
    await navigateTo(returnUrl)
  } catch (err: any) {
    const rawMessage = err.statusMessage || err.message || t('auth.login.error.invalidCredentials')
    errorMsg.value = sanitizeInput(rawMessage)
  } finally {
    isLoading.value = false
  }
}

async function handleOtpVerify() {
  if (form.otp_code.length !== 6) {
    errorMsg.value = t('auth.login.otp.invalidCode')
    return
  }
  isLoading.value = true
  errorMsg.value = ''

  try {
    await authStore.login({
      email: form.email,
      password: form.password,
      otp_code: form.otp_code,
    })
    toast.success(t('common.success'))
    await navigateTo(authStore.returnUrl || '/admin/dashboard')
    authStore.returnUrl = null
  } catch (err: any) {
    const rawMessage = err.statusMessage || t('auth.login.otp.invalidCode')
    errorMsg.value = sanitizeInput(rawMessage)
  } finally {
    isLoading.value = false
  }
}
</script>
