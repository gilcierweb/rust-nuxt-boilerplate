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

    <form v-else novalidate class="space-y-4" @submit.prevent="onSubmit">
      <div v-if="errorMsg" class="alert alert-error alert-soft" role="alert">
        <Icon name="heroicons:exclamation-circle" class="h-5 w-5" />
        <p class="text-sm">{{ errorMsg }}</p>
      </div>

      <div>
        <label class="label-text mb-1.5 block" for="password">{{ $t('auth.resetPassword.newPassword') }}</label>
        <div class="relative">
          <Icon name="heroicons:lock-closed" class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 opacity-50 pointer-events-none" />
          <input
            id="password"
            v-model="password"
            :type="showPw ? 'text' : 'password'"
            autocomplete="new-password"
            :placeholder="$t('auth.resetPassword.newPasswordPlaceholder')"
            class="input input-lg pl-10 w-full"
            :class="{ 'is-invalid': errors.password }"
          />
          <button
            type="button"
            class="absolute right-3 top-1/2 -translate-y-1/2 text-base-content/50 hover:text-base-content transition-colors"
            @click="showPw = !showPw"
          >
            <Icon :name="showPw ? 'heroicons:eye-slash' : 'heroicons:eye'" class="h-4 w-4" />
          </button>
        </div>
        <span v-if="errors.password" class="text-error text-xs mt-1 block">{{ errors.password }}</span>
      </div>

      <div>
        <label class="label-text mb-1.5 block" for="confirm">{{ $t('auth.resetPassword.confirmPassword') }}</label>
        <div class="relative">
          <Icon name="heroicons:lock-closed" class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 opacity-50 pointer-events-none" />
          <input
            id="confirm"
            v-model="password_confirmation"
            :type="showPw ? 'text' : 'password'"
            autocomplete="new-password"
            :placeholder="$t('auth.resetPassword.confirmPasswordPlaceholder')"
            class="input input-lg pl-10 w-full"
            :class="{ 'is-invalid': errors.password_confirmation }"
          />
        </div>
        <span v-if="errors.password_confirmation" class="text-error text-xs mt-1 block">{{ errors.password_confirmation }}</span>
      </div>

      <button type="submit" :disabled="loading" class="btn btn-primary btn-lg btn-gradient btn-block">
        <Icon v-if="loading" name="svg-spinners:3-dots-fade" class="h-5 w-5" />
        <template v-else>{{ $t('auth.resetPassword.submit') }}</template>
      </button>
    </form>
  </div>
</template>

<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/valibot'
import { useForm } from 'vee-validate'
import { mapAuthError } from '~/utils/auth-errors'
import { resetPasswordSchema, type ResetPasswordValues } from '~/forms/auth-schemas'

definePageMeta({ layout: 'auth' })

const { t } = useI18n()
const { $api } = useNuxtApp()
const localePath = useLocalePath()
const route = useRoute()
const token = route.query.token as string | undefined

const showPw = ref(false)
const loading = ref(false)
const success = ref(false)
const errorMsg = ref('')

const schema = computed(() => toTypedSchema(resetPasswordSchema(t)))
const { handleSubmit, errors, defineField } = useForm<ResetPasswordValues>({
  validationSchema: schema,
  initialValues: { password: '', password_confirmation: '' },
})

const [password] = defineField('password')
const [password_confirmation] = defineField('password_confirmation')

const onSubmit = handleSubmit(async (values) => {
  if (values.password !== values.password_confirmation) {
    errorMsg.value = t('auth.validation.passwordMismatch')
    return
  }
  loading.value = true
  errorMsg.value = ''
  try {
    await $api('/auth/reset', {
      method: 'POST',
      body: {
        token,
        password: values.password,
        password_confirmation: values.password_confirmation,
      },
    })
    success.value = true
  } catch (err: any) {
    errorMsg.value = mapAuthError(err, t, 'auth.resetPassword.error.invalidToken')
  } finally {
    loading.value = false
  }
})
</script>
