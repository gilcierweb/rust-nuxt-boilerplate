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
      <h3 class="mb-1.5 text-2xl font-semibold text-base-content">{{ $t('auth.register.title') }}</h3>
      <p class="text-base-content/80">
        {{ $t('auth.register.hasAccount') }}
        <NuxtLink :to="localePath('/auth/login')" class="link link-animated link-primary font-normal">
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
          {{ $t('auth.register.success.message', { email: email }) }}
        </p>
      </div>
      <NuxtLink
        :to="localePath('/auth/login')"
        class="btn btn-primary btn-gradient btn-block"
      >
        {{ $t('auth.register.success.goToLogin') }}
      </NuxtLink>
    </div>

    <form v-else novalidate class="space-y-4" @submit.prevent="onSubmit">
      <AppAlert
        v-if="errorMsg"
        tone="error"
        variant="soft"
        :message="errorMsg"
        :dismissible="false"
        class="text-sm"
      />

      <div>
        <label class="label-text" for="email">{{ $t('auth.register.email') }}*</label>
        <input
          id="email"
          v-model="email"
          type="email"
          autocomplete="email"
          :placeholder="$t('auth.register.emailPlaceholder')"
          :disabled="isLoading"
          class="input w-full"
          :class="{ 'is-invalid': errors.email }"
        />
        <span v-if="errors.email" class="text-error text-xs mt-1 block">{{ errors.email }}</span>
      </div>

      <div>
        <label class="label-text" for="password">{{ $t('auth.register.password') }}*</label>
        <div class="relative">
          <input
            id="password"
            v-model="password"
            :type="showPassword ? 'text' : 'password'"
            autocomplete="new-password"
            :placeholder="$t('auth.register.passwordPlaceholder')"
            :disabled="isLoading"
            class="input w-full pr-12"
            :class="{ 'is-invalid': errors.password }"
          />
          <button type="button" class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/50 hover:text-base-content transition-colors" aria-label="toggle password visibility" @click="showPassword = !showPassword">
            <span :class="[showPassword ? 'hidden' : 'block', 'icon-[tabler--eye] size-5 shrink-0']" />
            <span :class="[showPassword ? 'block' : 'hidden', 'icon-[tabler--eye-off] size-5 shrink-0']" />
          </button>
        </div>
        <span v-if="errors.password" class="text-error text-xs mt-1 block">{{ errors.password }}</span>
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
          v-model="password_confirmation"
          :type="showPassword ? 'text' : 'password'"
          autocomplete="new-password"
          :placeholder="$t('auth.register.confirmPasswordPlaceholder')"
          :disabled="isLoading"
          class="input w-full"
          :class="{ 'is-invalid': errors.password_confirmation }"
        />
        <span v-if="errors.password_confirmation" class="text-error text-xs mt-1 block">{{ errors.password_confirmation }}</span>
      </div>

      <div class="rounded-lg border border-base-300 bg-base-100 p-4">
        <label class="flex items-start gap-3">
          <input v-model="age_confirmed" type="checkbox" class="checkbox checkbox-primary checkbox-sm mt-0.5" :class="{ 'is-invalid': errors.age_confirmed }" />
          <p class="text-sm text-base-content/80 leading-relaxed">
            {{ $t('auth.register.terms.consent') }}
            <NuxtLink :to="localePath('/terms')" class="link link-primary font-normal">{{ $t('auth.register.terms.termsOfUse') }}</NuxtLink>
            {{ $t('auth.register.terms.and') }}
            <NuxtLink :to="localePath('/privacy')" class="link link-primary font-normal">{{ $t('auth.register.terms.privacyPolicy') }}</NuxtLink>.
          </p>
        </label>
        <span v-if="errors.age_confirmed" class="text-error text-xs mt-1 block">{{ errors.age_confirmed }}</span>
      </div>

      <button
        type="submit"
        :disabled="isLoading || !meta.valid"
        class="btn btn-lg btn-primary btn-gradient btn-block disabled:opacity-60"
      >
        <Icon v-if="isLoading" name="svg-spinners:ring-resize" class="h-5 w-5" />
        <template v-else>{{ $t('auth.register.submit') }}</template>
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/valibot'
import { useForm } from 'vee-validate'
import { mapAuthError } from '~/utils/auth-errors'
import { registerSchema, type RegisterValues } from '~/forms/auth-schemas'

definePageMeta({
  layout: 'auth',
})

const { t } = useI18n()
const authStore = useAuthStore()
const localePath = useLocalePath()

const isLoading = ref(false)
const showPassword = ref(false)
const errorMsg = ref('')
const success = ref(false)

const schema = computed(() => toTypedSchema(registerSchema(t)))
const { handleSubmit, errors, meta, defineField } = useForm<RegisterValues>({
  validationSchema: schema,
  initialValues: { email: '', password: '', password_confirmation: '', age_confirmed: false },
})

const [email] = defineField('email')
const [password] = defineField('password')
const [password_confirmation] = defineField('password_confirmation')
const [age_confirmed] = defineField('age_confirmed')

const passwordStrength = computed(() => {
  const p = password.value
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

const onSubmit = handleSubmit(async (values) => {
  errorMsg.value = ''
  isLoading.value = true
  try {
    await authStore.register({
      email: values.email,
      password: values.password,
      password_confirmation: values.password_confirmation,
    })
    success.value = true
  } catch (err: any) {
    errorMsg.value = mapAuthError(err, t, 'auth.register.error.generic')
  } finally {
    isLoading.value = false
  }
})
</script>
