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
      <h3 class="mb-1.5 text-2xl font-semibold text-base-content">{{ $t('auth.magicLink.title') }}</h3>
      <p class="text-base-content/80">{{ $t('auth.magicLink.description') }}</p>
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

    <form v-if="!isLoading || !successMsg" novalidate class="space-y-4" @submit.prevent="onSubmit">
      <div>
        <label class="label-text" for="email">{{ $t('auth.login.email') }}*</label>
        <input
          id="email"
          v-model="email"
          type="email"
          autocomplete="email"
          :placeholder="$t('auth.login.emailPlaceholder')"
          :disabled="isLoading"
          class="input w-full"
          :class="{ 'is-invalid': errors.email }"
        />
        <span v-if="errors.email" class="text-error text-xs mt-1 block">{{ errors.email }}</span>
      </div>

      <button type="submit" :disabled="isLoading" class="btn btn-lg btn-primary btn-gradient btn-block">
        <span v-if="isLoading" class="icon-[tabler--loader-2] size-5 animate-spin"></span>
        <template v-else>{{ $t('auth.magicLink.sendButton') }}</template>
      </button>
    </form>

    <p class="text-center text-base-content/80">
      {{ $t('auth.login.backToLogin') }}
      <NuxtLink :to="localePath('/auth/login')" class="link link-animated link-primary font-normal">
        {{ $t('auth.login.submit') }}
      </NuxtLink>
    </p>
  </div>
</template>

<script setup lang="ts">
import { toTypedSchema } from '@vee-validate/valibot'
import { useForm } from 'vee-validate'
import { useAuthStore } from '~/stores/auth'
import { mapAuthError } from '~/utils/auth-errors'
import { z } from 'zod'

definePageMeta({
  layout: 'auth',
})

const { t } = useI18n()
const authStore = useAuthStore()
const localePath = useLocalePath()

const isLoading = ref(false)
const errorMsg = ref('')
const successMsg = ref('')

const schema = computed(() => toTypedSchema(
  z.object({
    email: z.string().email(t('auth.validation.invalidEmail')).max(254, t('auth.validation.emailTooLong')),
  })
))

const { handleSubmit, errors, defineField, resetForm } = useForm({
  validationSchema: schema,
  initialValues: { email: '' },
})

const [email] = defineField('email')

const onSubmit = handleSubmit(async (values) => {
  errorMsg.value = ''
  successMsg.value = ''
  isLoading.value = true
  try {
    await authStore.requestMagicLink(values.email)
    successMsg.value = t('auth.magicLink.sent')
    resetForm()
  } catch (err: any) {
    errorMsg.value = mapAuthError(err, t, 'auth.magicLink.invalid')
  } finally {
    isLoading.value = false
  }
})
</script>