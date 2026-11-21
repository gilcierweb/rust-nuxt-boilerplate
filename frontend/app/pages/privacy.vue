<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-16 lg:py-24">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 relative z-10">
          <div class="max-w-3xl">
            <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
              <span class="icon-[tabler--shield] text-sm"></span>
              {{ $t('landing.auto.privacy.1') }}
            </div>
            <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight">
              {{ $t('landing.auto.privacy.2') }}
            </h1>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
              {{ $t('landing.auto.privacy.3') }}
            </p>
          </div>
        </div>
      </section>

      <!-- Content Section -->
      <section class="py-12 md:py-20 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-4 gap-6 lg:gap-12">
            <!-- Sidebar Navigation -->
            <div class="hidden lg:block lg:col-span-1">
              <div class="sticky top-24 bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <h3 class="font-headline-sm text-headline-sm text-on-surface mb-4">{{ $t('landing.auto.privacy.4') }}</h3>
                <nav class="space-y-2">
                  <a href="#coleta" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.5') }}
                  </a>
                  <a href="#uso" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.6') }}
                  </a>
                  <a href="#compartilhamento" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.7') }}
                  </a>
                  <a href="#seguranca" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.8') }}
                  </a>
                  <a href="#direitos" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.9') }}
                  </a>
                  <a href="#cookies" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.10') }}
                  </a>
                  <a href="#contato" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.privacy.11') }}
                  </a>
                </nav>
              </div>
            </div>

            <!-- Main Content -->
            <div class="lg:col-span-3">
              <div class="bg-surface-container-lowest p-5 md:p-8 lg:p-10 rounded-2xl border border-outline-variant/30 space-y-8 md:space-y-12">
                <!-- Intro -->
                <div class="prose prose-lg max-w-none">
                  <p class="font-body-lg text-body-lg text-on-surface-variant leading-relaxed">
                    {{ $t('landing.auto.privacy.12') }}
                  </p>
                </div>

                <!-- Section 1 -->
                <div id="coleta" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary flex-shrink-0">
                      <span class="icon-[tabler--database]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.13') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.14') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--circle-check] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.15') }}</strong> {{ $t('landing.auto.privacy.16') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--circle-check] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.17') }}</strong> {{ $t('landing.auto.privacy.18') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--circle-check] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.19') }}</strong> {{ $t('landing.auto.privacy.20') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--circle-check] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.21') }}</strong> {{ $t('landing.auto.privacy.22') }}</span>
                      </li>
                    </ul>
                  </div>
                </div>

                <!-- Section 2 -->
                <div id="uso" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary">
                      <span class="icon-[tabler--apps]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.23') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.24') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-secondary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.25') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-secondary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.26') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-secondary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.27') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-secondary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.28') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-secondary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.29') }}</span>
                      </li>
                    </ul>
                  </div>
                </div>

                <!-- Section 3 -->
                <div id="compartilhamento" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-tertiary-container/10 flex items-center justify-center text-tertiary">
                      <span class="icon-[tabler--share]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.30') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.31') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-tertiary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.32') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-tertiary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.33') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-tertiary mt-0.5"></span>
                        <span>{{ $t('landing.auto.privacy.34') }}</span>
                      </li>
                    </ul>
                  </div>
                </div>

                <!-- Section 4 -->
                <div id="seguranca" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-success-container/10 flex items-center justify-center text-success">
                      <span class="icon-[tabler--shield-lock]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.35') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.36') }}
                    </p>
                    <div class="grid md:grid-cols-2 gap-4">
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--lock] text-success mb-2"></span>
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.privacy.37') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.38') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--user-check] text-success mb-2"></span>
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.privacy.39') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.40') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--database-export] text-success mb-2"></span>
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.privacy.41') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.42') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--activity] text-success mb-2"></span>
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.privacy.43') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.44') }}</p>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Section 5 -->
                <div id="direitos" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary">
                      <span class="icon-[tabler--gavel]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.45') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.46') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--eye] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.47') }}</strong> {{ $t('landing.auto.privacy.48') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--edit] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.49') }}</strong> {{ $t('landing.auto.privacy.50') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--trash] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.51') }}</strong> {{ $t('landing.auto.privacy.52') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--ban] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.53') }}</strong> {{ $t('landing.auto.privacy.54') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--download] text-primary mt-0.5"></span>
                        <span><strong class="text-on-surface">{{ $t('landing.auto.privacy.55') }}</strong> {{ $t('landing.auto.privacy.56') }}</span>
                      </li>
                    </ul>
                  </div>
                </div>

                <!-- Section 6 -->
                <div id="cookies" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary">
                      <span class="icon-[tabler--cookie]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.57') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.58') }}
                    </p>
                    <div class="space-y-3">
                      <div class="flex items-start gap-4 p-4 bg-surface rounded-xl border border-outline-variant/20">
                        <div class="w-8 h-8 rounded-lg bg-primary/10 flex items-center justify-center flex-shrink-0">
                          <span class="icon-[tabler--check] text-primary text-sm"></span>
                        </div>
                        <div>
                          <h4 class="font-headline-sm text-headline-sm text-on-surface">{{ $t('landing.auto.privacy.59') }}</h4>
                          <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.60') }}</p>
                        </div>
                      </div>
                      <div class="flex items-start gap-4 p-4 bg-surface rounded-xl border border-outline-variant/20">
                        <div class="w-8 h-8 rounded-lg bg-secondary/10 flex items-center justify-center flex-shrink-0">
                          <span class="icon-[tabler--check] text-secondary text-sm"></span>
                        </div>
                        <div>
                          <h4 class="font-headline-sm text-headline-sm text-on-surface">{{ $t('landing.auto.privacy.61') }}</h4>
                          <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.62') }}</p>
                        </div>
                      </div>
                      <div class="flex items-start gap-4 p-4 bg-surface rounded-xl border border-outline-variant/20">
                        <div class="w-8 h-8 rounded-lg bg-tertiary/10 flex items-center justify-center flex-shrink-0">
                          <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                        </div>
                        <div>
                          <h4 class="font-headline-sm text-headline-sm text-on-surface">{{ $t('landing.auto.privacy.63') }}</h4>
                          <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.privacy.64') }}</p>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Section 7 -->
                <div id="contato" class="scroll-mt-24">
                  <div class="flex items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-tertiary-container/10 flex items-center justify-center text-tertiary">
                      <span class="icon-[tabler--help-circle]"></span>
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.privacy.65') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.privacy.66') }}
                    </p>
                    <div class="bg-surface p-6 rounded-xl border border-outline-variant/20">
                      <div class="space-y-3">
                        <div class="flex items-center gap-3">
                          <span class="icon-[tabler--mail] text-tertiary"></span>
                          <span class="font-body-md text-body-md text-on-surface">{{ $t('landing.auto.privacy.67') }}</span>
                        </div>
                        <div class="flex items-center gap-3">
                          <span class="icon-[tabler--phone] text-tertiary"></span>
                          <span class="font-body-md text-body-md text-on-surface">(11) 99999-9999</span>
                        </div>
                        <div class="flex items-center gap-3">
                          <span class="icon-[tabler--map-pin] text-tertiary"></span>
                          <span class="font-body-md text-body-md text-on-surface">{{ $t('landing.auto.privacy.68') }}</span>
                        </div>
                      </div>
                    </div>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">
                      {{ $t('landing.auto.privacy.69') }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- Back to Top -->
              <div class="mt-8 flex justify-center">
                <button 
                  class="btn btn-ghost gap-2"
                  @click="scrollToTop"
                >
                  <span class="icon-[tabler--arrow-up]"></span>
                  {{ $t('landing.auto.privacy.70') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-16 bg-surface-container-lowest border-t border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-6 text-center">
          <p class="font-body-md text-body-md text-on-surface-variant mb-4">
            {{ $t('landing.auto.privacy.71') }}
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <NuxtLink to="/terms" class="btn btn-outline btn-primary">
              <span class="icon-[tabler--file-text] mr-2"></span>
              {{ $t('landing.auto.privacy.72') }}
            </NuxtLink>
            <NuxtLink to="/contact" class="btn btn-ghost">
              <span class="icon-[tabler--help-circle] mr-2"></span>
              {{ $t('landing.auto.privacy.73') }}
            </NuxtLink>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup>
definePageMeta({
  layout: 'landing'
})

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>