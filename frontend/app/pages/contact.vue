<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="icon-[tabler--mail] text-sm"></span>
            {{ $t('landing.auto.contact.1') }}
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            {{ $t('landing.auto.contact.2') }}
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            {{ $t('landing.auto.contact.3') }}
          </p>
        </div>
      </section>

      <!-- Contact Grid -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-3 gap-6 lg:gap-8">
            <!-- Contact Info Cards -->
            <div class="lg:col-span-1 space-y-6">
              <!-- Email -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                  <span class="icon-[tabler--mail] text-2xl"></span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.contact.4') }}</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  {{ $t('landing.auto.contact.5') }}
                </p>
                <a href="mailto:hello@boilerplate.dev" class="text-primary font-medium hover:underline">
                  {{ $t('landing.auto.contact.6') }}
                </a>
              </div>

              <!-- GitHub -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                  <span class="icon-[tabler--code] text-2xl"></span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.contact.7') }}</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  {{ $t('landing.auto.contact.8') }}
                </p>
                <a href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank" class="text-secondary font-medium hover:underline flex items-center gap-1">
                  <span class="icon-[tabler--external-link] text-sm"></span>
                  {{ $t('landing.auto.contact.9') }}
                </a>
              </div>

              <!-- Discord -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                  <span class="icon-[tabler--message] text-2xl"></span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.contact.10') }}</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  {{ $t('landing.auto.contact.11') }}
                </p>
                <a href="https://discord.gg/your-invite" target="_blank" class="text-tertiary font-medium hover:underline flex items-center gap-1">
                  <span class="icon-[tabler--external-link] text-sm"></span>
                  {{ $t('landing.auto.contact.12') }}
                </a>
              </div>
            </div>

            <!-- Contact Form -->
            <div class="lg:col-span-2">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30">
                <h2 class="font-headline-lg text-headline-lg text-on-surface mb-6">
                  {{ $t('landing.auto.contact.13') }}
                </h2>
                
                <form class="space-y-6" @submit.prevent="handleSubmit">
                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Name -->
                    <div class="form-control">
                      <label class="label-text font-semibold" for="contactName">
                        {{ $t('landing.auto.contact.14') }} <span class="text-error">*</span>
                      </label>
                      <input
                        id="contactName"
                        v-model="form.name"
                        type="text"
                        placeholder="Your name"
                        class="input w-full"
                        required
                      />
                    </div>

                    <!-- Email -->
                    <div class="form-control">
                      <label class="label-text font-semibold" for="contactEmail">
                        {{ $t('landing.auto.contact.15') }} <span class="text-error">*</span>
                      </label>
                      <input
                        id="contactEmail"
                        v-model="form.email"
                        type="email"
                        placeholder="you@example.com"
                        class="input w-full"
                        required
                      />
                    </div>
                  </div>

                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Company/Org (optional) -->
                    <div class="form-control">
                      <label class="label-text font-semibold" for="contactCompany">{{ $t('landing.auto.contact.16') }}</label>
                      <input
                        id="contactCompany"
                        v-model="form.company"
                        type="text"
                        placeholder="Your company (optional)"
                        class="input w-full"
                      />
                    </div>

                    <!-- Subject -->
                    <div class="form-control">
                      <label class="label-text font-semibold" for="contactSubject">
                        {{ $t('landing.auto.contact.17') }} <span class="text-error">*</span>
                      </label>
                      <select id="contactSubject" v-model="form.subject" class="select w-full" required>
                        <option value="">{{ $t('landing.auto.contact.18') }}</option>
                        <option value="general">{{ $t('landing.auto.contact.19') }}</option>
                        <option value="support">{{ $t('landing.auto.contact.20') }}</option>
                        <option value="enterprise">{{ $t('landing.auto.contact.21') }}</option>
                        <option value="security">{{ $t('landing.auto.contact.22') }}</option>
                        <option value="contribution">{{ $t('landing.auto.contact.23') }}</option>
                        <option value="other">{{ $t('landing.auto.contact.24') }}</option>
                      </select>
                    </div>
                  </div>

                  <!-- Message -->
                  <div class="form-control">
                    <label class="label-text font-semibold" for="contactMessage">
                      {{ $t('landing.auto.contact.25') }} <span class="text-error">*</span>
                    </label>
                    <textarea
                      id="contactMessage"
                      v-model="form.message"
                      placeholder="How can we help you?"
                      class="textarea w-full min-h-[150px]"
                      required
                    ></textarea>
                  </div>

                  <!-- Privacy Checkbox -->
                  <div class="form-control">
                    <label class="flex items-start gap-3 cursor-pointer">
                      <input v-model="form.privacy" type="checkbox" class="checkbox checkbox-primary" required />
                      <span class="label-text text-on-surface-variant">
                        {{ $t('landing.auto.contact.26') }} <NuxtLink to="/privacy" class="text-primary hover:underline">{{ $t('landing.auto.contact.27') }}</NuxtLink> {{ $t('landing.auto.contact.28') }}
                      </span>
                    </label>
                  </div>

                  <!-- Submit -->
                  <div class="flex items-center gap-4 pt-4">
                    <button
                      type="submit"
                      class="btn btn-primary px-8"
                      :disabled="submitting"
                    >
                      <span v-if="submitting" class="loading loading-spinner loading-sm mr-2"></span>
                      <span v-else class="icon-[tabler--send] mr-2"></span>
                      {{ submitting ? 'Sending...' : 'Send Message' }}
                    </button>
                    
                    <span v-if="success" class="text-success flex items-center gap-2">
                      <span class="icon-[tabler--circle-check]"></span>
                      {{ $t('landing.auto.contact.29') }}
                    </span>
                    
                    <span v-if="error" class="text-error flex items-center gap-2">
                      <span class="icon-[tabler--alert-circle]"></span>
                      {{ $t('landing.auto.contact.30') }}
                    </span>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.contact.31') }}</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              {{ $t('landing.auto.contact.32') }}
            </p>
          </div>

          <div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
            <!-- FAQ 1 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-primary"></span>
                {{ $t('landing.auto.contact.33') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.34') }}
              </p>
            </div>

            <!-- FAQ 2 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-secondary"></span>
                {{ $t('landing.auto.contact.35') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.36') }}
              </p>
            </div>

            <!-- FAQ 3 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-tertiary"></span>
                {{ $t('landing.auto.contact.37') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.38') }}
              </p>
            </div>

            <!-- FAQ 4 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-success"></span>
                {{ $t('landing.auto.contact.39') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.40') }}
              </p>
            </div>

            <!-- FAQ 5 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-warning"></span>
                {{ $t('landing.auto.contact.41') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.42') }}
              </p>
            </div>

            <!-- FAQ 6 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-info"></span>
                {{ $t('landing.auto.contact.43') }}
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.contact.44') }}
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Support Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.contact.45') }}</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                {{ $t('landing.auto.contact.46') }}
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink to="/contact" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  <span class="icon-[tabler--briefcase]"></span>
                  {{ $t('landing.auto.contact.47') }}
                </NuxtLink>
                <NuxtLink to="https://github.com/gilcierweb/rust-nuxt-boilerplate/discussions" target="_blank" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  <span class="icon-[tabler--messages]"></span>
                  {{ $t('landing.auto.contact.48') }}
                </NuxtLink>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'landing'
})

const form = reactive({
  name: '',
  email: '',
  company: '',
  subject: '',
  message: '',
  privacy: false
})

const submitting = ref(false)
const success = ref(false)
const error = ref(false)

async function handleSubmit() {
  submitting.value = true
  error.value = false
  
  // Simulate API call
  await new Promise(resolve => setTimeout(resolve, 1500))
  
  submitting.value = false
  success.value = true
  
  // Reset form
  Object.keys(form).forEach(key => {
    form[key] = key === 'privacy' ? false : ''
  })
  
  // Reset success message after 5 seconds
  setTimeout(() => {
    success.value = false
  }, 5000)
}
</script>