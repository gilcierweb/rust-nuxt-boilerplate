<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div
            class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div
              class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="icon-[tabler--info-circle] text-sm"></span>
            {{ $t('landing.auto.about.1') }}
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            {{ $t('landing.auto.about.2') }}
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            {{ $t('landing.auto.about.3') }}
          </p>
        </div>
      </section>

      <!-- Mission, Vision, Values -->
      <section class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid md:grid-cols-3 gap-6 lg:gap-8">
            <!-- Mission -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--rocket] text-3xl"></span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{{ $t('landing.auto.about.4') }}</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.about.5') }}
              </p>
            </div>

            <!-- Vision -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--eye] text-3xl"></span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{{ $t('landing.auto.about.6') }}</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                {{ $t('landing.auto.about.7') }}
              </p>
            </div>

            <!-- Values -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--circle-check] text-3xl"></span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{{ $t('landing.auto.about.8') }}</h3>
              <ul class="space-y-2 font-body-md text-body-md text-on-surface-variant">
                <li class="flex items-center gap-2">
                  <span class="icon-[tabler--check] text-primary text-sm"></span>
                  {{ $t('landing.auto.about.9') }}
                </li>
                <li class="flex items-center gap-2">
                  <span class="icon-[tabler--check] text-primary text-sm"></span>
                  {{ $t('landing.auto.about.10') }}
                </li>
                <li class="flex items-center gap-2">
                  <span class="icon-[tabler--check] text-primary text-sm"></span>
                  {{ $t('landing.auto.about.11') }}
                </li>
                <li class="flex items-center gap-2">
                  <span class="icon-[tabler--check] text-primary text-sm"></span>
                  {{ $t('landing.auto.about.12') }}
                </li>
                <li class="flex items-center gap-2">
                  <span class="icon-[tabler--check] text-primary text-sm"></span>
                  {{ $t('landing.auto.about.13') }}
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      <!-- Timeline / Journey -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.about.14') }}</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              {{ $t('landing.auto.about.15') }}
            </p>
          </div>

          <div class="relative">
            <!-- Timeline Line -->
            <div
                class="absolute left-1/2 transform -translate-x-1/2 h-full w-0.5 bg-outline-variant/50 hidden md:block"></div>

            <div class="space-y-12">
              <!-- 2024 Q1 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">{{ $t('landing.auto.about.16') }}</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">{{ $t('landing.auto.about.17') }}</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      {{ $t('landing.auto.about.18') }}
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2024 Q2 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-secondary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-secondary tracking-widest uppercase">{{ $t('landing.auto.about.19') }}</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">{{ $t('landing.auto.about.20') }}</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      {{ $t('landing.auto.about.21') }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2024 Q3 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-tertiary tracking-widest uppercase">{{ $t('landing.auto.about.22') }}</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">{{ $t('landing.auto.about.23') }}</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      {{ $t('landing.auto.about.24') }}
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-tertiary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2024 Q4 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">{{ $t('landing.auto.about.25') }}</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">{{ $t('landing.auto.about.26') }}</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      {{ $t('landing.auto.about.27') }}
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2025 Q1 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-success tracking-widest uppercase">{{ $t('landing.auto.about.28') }}</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">{{ $t('landing.auto.about.29') }}</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      {{ $t('landing.auto.about.30') }}
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-success border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Architecture Principles -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.about.31') }}</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              {{ $t('landing.auto.about.32') }}
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--shield] text-4xl text-primary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.33') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.34') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-secondary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--code] text-4xl text-secondary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.35') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.36') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-tertiary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--topology-star] text-4xl text-tertiary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.37') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.38') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--bolt] text-4xl text-primary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.39') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.40') }}
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Tech Stack -->
      <section class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.about.41') }}</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              {{ $t('landing.auto.about.42') }}
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <span class="icon-[tabler--brand-rust] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.43') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.44') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <span class="icon-[tabler--database-cog] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.45') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.46') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                <span class="icon-[tabler--database] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.47') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.48') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <span class="icon-[tabler--server] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.49') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.50') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <span class="icon-[tabler--brand-nuxt] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.51') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.52') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                <span class="icon-[tabler--brand-tailwind] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.53') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.54') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <span class="icon-[tabler--stack-2] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.55') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.56') }}
</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <span class="icon-[tabler--brand-docker] w-8 h-8"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{{ $t('landing.auto.about.57') }}</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">
{{ $t('landing.auto.about.58') }}
</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Team / Contributors -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.about.59') }}</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              {{ $t('landing.auto.about.60') }}
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--user] text-4xl text-primary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.61') }}</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">{{ $t('landing.auto.about.62') }}</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.63') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-secondary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--users] text-4xl text-secondary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.64') }}</h3>
              <p class="font-body-sm text-body-sm text-secondary mt-1">{{ $t('landing.auto.about.65') }}</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.66') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-tertiary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--bug] text-4xl text-tertiary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.67') }}</h3>
              <p class="font-body-sm text-body-sm text-tertiary mt-1">{{ $t('landing.auto.about.68') }}</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.69') }}
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="icon-[tabler--heart] text-4xl text-primary"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">{{ $t('landing.auto.about.70') }}</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">{{ $t('landing.auto.about.71') }}</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                {{ $t('landing.auto.about.72') }}
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Stats Section -->
      <section class="py-16 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-6 lg:gap-8">
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-primary mb-2">15+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{{ $t('landing.auto.about.73') }}</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-secondary mb-2">3</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{{ $t('landing.auto.about.74') }}</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-tertiary mb-2">100%</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{{ $t('landing.auto.about.75') }}</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-success mb-2">{{ $t('landing.auto.about.76') }}</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{{ $t('landing.auto.about.77') }}</div>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div
              class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>

            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{{ $t('landing.auto.about.78') }}</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                {{ $t('landing.auto.about.79') }}
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink
to="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank"
                          class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  {{ $t('landing.auto.about.80') }} <span class="icon-[tabler--arrow-right]"></span>
                </NuxtLink>
                <NuxtLink
to="/contact"
                          class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  {{ $t('landing.auto.about.81') }}
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
</script>

<style>

</style>