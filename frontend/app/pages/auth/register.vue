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
      <h3 class="mb-1.5 text-2xl font-semibold text-base-content">{{ $t('auth.register.title') }}</h3>
      <p class="text-base-content/80">
        {{ $t('auth.register.hasAccount') }}
        <NuxtLink to="/auth/login" class="link link-animated link-primary font-normal">
          {{ $t('auth.register.login') }}
        </NuxtLink>
      </p>
    </div>

    <div
      v-if="success"
      class="space-y-4 rounded-xl border border-success/20 bg-success/5 p-6 text-center"
    >
      <div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-success/15">
        <Icon name="heroicons:envelope-open-solid" class="h-7 w-7 text-success" />
      </div>
      <div class="space-y-1">
        <h3 class="text-xl font-semibold text-base-content">{{ $t('auth.register.success.title') }}</h3>
        <p class="text-sm text-base-content/70">
          {{ $t('auth.register.success.message', { email: form.email }) }}
        </p>
      </div>
      <NuxtLink
        to="/auth/login"
        class="btn btn-primary btn-gradient btn-block"
      >
        {{ $t('auth.register.success.goToLogin') }}
      </NuxtLink>
    </div>

    <form v-else class="space-y-4" @submit.prevent="handleRegister">
      <Transition enter-active-class="duration-300 ease-out" enter-from-class="opacity-0 -translate-y-2">
        <div v-if="errorMsg" class="alert alert-error alert-soft text-sm">
          <Icon name="heroicons:exclamation-circle" class="h-5 w-5" />
          <span>{{ errorMsg }}</span>
        </div>
      </Transition>

      <div>
        <label class="label-text" for="email">{{ $t('auth.register.email') }}*</label>
        <input
          id="email"
          v-model="form.email"
          type="email"
          required
          autocomplete="email"
          :placeholder="$t('auth.register.emailPlaceholder')"
          :disabled="isLoading"
          class="input"
        />
      </div>

      <div>
        <label class="label-text" for="password">{{ $t('auth.register.password') }}*</label>
        <div class="input">
          <input
            id="password"
            v-model="form.password"
            :type="showPassword ? 'text' : 'password'"
            required
            autocomplete="new-password"
            :placeholder="$t('auth.register.passwordPlaceholder')"
            :disabled="isLoading"
          />
          <button type="button" class="block cursor-pointer" aria-label="toggle password visibility" @click="showPassword = !showPassword">
            <span :class="[showPassword ? 'hidden' : 'block', 'icon-[tabler--eye] size-5 shrink-0']" />
            <span :class="[showPassword ? 'block' : 'hidden', 'icon-[tabler--eye-off] size-5 shrink-0']" />
          </button>
        </div>
        <div class="space-y-2 pt-2">
          <div class="flex gap-2">
            <div v-for="i in 4" :key="i" :class="['h-1.5 flex-1 rounded-full transition-all', passwordStrength >= i ? strengthColor : 'bg-base-300']" />
          </div>
          <p class="text-xs font-medium" :class="strengthTextColor">
            {{ strengthLabel || $t('auth.register.errors.setStrongPassword') }}
          </p>
        </div>
      </div>

      <div>
        <label class="label-text" for="password_confirmation">{{ $t('auth.register.confirmPassword') }}*</label>
        <input
          id="password_confirmation"
          v-model="form.password_confirmation"
          :type="showPassword ? 'text' : 'password'"
          required
          autocomplete="new-password"
          :placeholder="$t('auth.register.confirmPasswordPlaceholder')"
          :disabled="isLoading"
          :class="['input', passwordMismatch ? 'input-error' : '']"
        />
        <p v-if="passwordMismatch" class="mt-1 text-xs text-error">
          {{ $t('auth.register.errors.passwordMismatch') }}
        </p>
      </div>

      <div class="rounded-lg border border-base-300 bg-base-100 p-4">
        <label class="flex items-start gap-3">
          <input type="checkbox" v-model="form.age_confirmed" required class="checkbox checkbox-primary checkbox-sm mt-0.5" />
          <p class="text-sm text-base-content/80 leading-relaxed">
            {{ $t('auth.register.terms.consent') }}
            <NuxtLink to="/terms" class="link link-primary font-normal">{{ $t('auth.register.terms.termsOfUse') }}</NuxtLink>
            {{ $t('auth.register.terms.and') }}
            <NuxtLink to="/privacy" class="link link-primary font-normal">{{ $t('auth.register.terms.privacyPolicy') }}</NuxtLink>.
          </p>
        </label>
      </div>

      <button
        type="submit"
        :disabled="isLoading || !form.age_confirmed || passwordMismatch"
        class="btn btn-lg btn-primary btn-gradient btn-block disabled:opacity-60"
      >
        <Icon v-if="isLoading" name="svg-spinners:ring-resize" class="h-5 w-5" />
        <template v-else>{{ $t('auth.register.submit') }}</template>
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'auth',
  
})

const { t } = useI18n()
const authStore = useAuthStore()

const form = reactive({
  email: '',
  password: '',
  password_confirmation: '',
  age_confirmed: false,
})

const isLoading = ref(false)
const showPassword = ref(false)
const errorMsg = ref('')
const success = ref(false)

const passwordMismatch = computed(
  () => form.password_confirmation.length > 0 && form.password !== form.password_confirmation,
)

const passwordStrength = computed(() => {
  const p = form.password
  if (!p) return 0
  let score = 0
  if (p.length >= 8) score++
  if (/[A-Z]/.test(p)) score++
  if (/[0-9]/.test(p)) score++
  if (/[^A-Za-z0-9]/.test(p)) score++
  return score
})

const strengthColor = computed(() => {
  const colors = ['', 'bg-rose-500', 'bg-orange-400', 'bg-yellow-400', 'bg-emerald-500']
  return colors[passwordStrength.value]
})

const strengthTextColor = computed(() => {
  const colors = ['', 'text-error', 'text-warning', 'text-warning', 'text-success']
  return colors[passwordStrength.value] || 'text-base-content/50'
})

const strengthLabel = computed(() => {
  const labels = ['', t('auth.register.strength.weak'), t('auth.register.strength.fair'), t('auth.register.strength.good'), t('auth.register.strength.strong')]
  return passwordStrength.value ? t('auth.register.strength.label', { strength: labels[passwordStrength.value] }) : ''
})

async function handleRegister() {
  if (passwordMismatch.value) return
  errorMsg.value = ''
  isLoading.value = true

  try {
    await authStore.register({
      email: form.email,
      password: form.password,
      password_confirmation: form.password_confirmation,
    })
    success.value = true
  } catch (err: any) {
    errorMsg.value = err.statusMessage || err.message || t('auth.register.error.generic')
  } finally {
    isLoading.value = false
  }
}
</script>
