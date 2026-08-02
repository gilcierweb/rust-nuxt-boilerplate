<template>
  <div>
    <!-- Title -->
    <div class="mb-6">
      <h3 class="text-base-content text-2xl font-semibold mb-1">{{ $t('auth.forgotPassword.title') }}</h3>
      <p class="text-base-content/70 text-sm">
        {{ $t('auth.forgotPassword.remembered') }}
        <NuxtLink :to="localePath('/auth/login')" class="link link-primary font-medium">
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

    <form v-else novalidate class="space-y-4" @submit.prevent="onSubmit">
      <p class="text-base-content/70 text-sm leading-relaxed">
        {{ $t('auth.forgotPassword.description') }}
      </p>

      <div>
        <label class="label-text mb-1.5 block" for="email">{{ $t('auth.forgotPassword.email') }}</label>
        <div class="relative">
          <Icon name="heroicons:envelope" class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 opacity-50 pointer-events-none" />
          <input
            id="email"
            v-model="email"
            type="email"
            autocomplete="email"
            :placeholder="$t('auth.forgotPassword.emailPlaceholder')"
            :disabled="loading"
            class="input input-lg pl-10 w-full"
            :class="{ 'is-invalid': errors.email }"
          />
        </div>
        <span v-if="errors.email" class="text-error text-xs mt-1 block">{{ errors.email }}</span>
      </div>

      <button type="submit" :disabled="loading || !meta.valid" class="btn btn-primary btn-lg btn-gradient btn-block">
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
import { toTypedSchema } from '@vee-validate/valibot'
import { useForm } from 'vee-validate'
import { forgotPasswordSchema, type ForgotPasswordValues } from '~/forms/auth-schemas'

definePageMeta({ layout: 'auth' })

const { t } = useI18n()
const { $api } = useNuxtApp()
const localePath = useLocalePath()

const loading = ref(false)
const sent = ref(false)

const schema = computed(() => toTypedSchema(forgotPasswordSchema(t)))
const { handleSubmit, errors, meta, defineField } = useForm<ForgotPasswordValues>({
  validationSchema: schema,
  initialValues: { email: '' },
})

const [email] = defineField('email')

const onSubmit = handleSubmit(async (values) => {
  loading.value = true
  try {
    await $api('/auth/forgot-password', { method: 'POST', body: { email: values.email } })
  } catch {}
  sent.value = true
  loading.value = false
})
</script>
