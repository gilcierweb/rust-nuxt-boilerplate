<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-16 lg:py-24">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 relative z-10">
          <div class="max-w-3xl">
            <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
              <span class="icon-[tabler--scale] text-sm" />
              {{ $t('landing.auto.terms.1') }}
            </div>
            <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight">
              {{ $t('landing.auto.terms.2') }}
            </h1>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
              {{ $t('landing.auto.terms.3') }}
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
                <h3 class="font-headline-sm text-headline-sm text-on-surface mb-4">{{ $t('landing.auto.terms.4') }}</h3>
                <nav class="space-y-2">
                  <a href="#aceitacao" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.5') }}
                  </a>
                  <a href="#conta" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.6') }}
                  </a>
                  <a href="#servicos" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.7') }}
                  </a>
                  <a href="#pagamento" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.8') }}
                  </a>
                  <a href="#cancelamento" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.9') }}
                  </a>
                  <a href="#propriedade" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.10') }}
                  </a>
                  <a href="#limitacao" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.11') }}
                  </a>
                  <a href="#contato" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    {{ $t('landing.auto.terms.12') }}
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
                    {{ $t('landing.auto.terms.13') }}
                  </p>
                </div>

                <!-- Section 1 -->
                <div id="aceitacao" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary flex-shrink-0">
                      <span class="icon-[tabler--circle-check]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.14') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.terms.15') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>{{ $t('landing.auto.terms.16') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>{{ $t('landing.auto.terms.17') }} <NuxtLink to="/privacy" class="text-primary hover:underline">{{ $t('landing.auto.terms.18') }}</NuxtLink></span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>{{ $t('landing.auto.terms.19') }}</span>
                      </li>
                    </ul>
                    <div class="bg-warning/10 border border-warning/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--alert-triangle] text-warning mr-2" />
                        {{ $t('landing.auto.terms.20') }}
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 2 -->
                <div id="conta" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary flex-shrink-0">
                      <span class="icon-[tabler--user]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.21') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.22') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.23') }}
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.24') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.25') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.26') }}</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.27') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.28') }}
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.29') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.30') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.31') }}</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                </div>

                <!-- Section 3 -->
                <div id="servicos" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-tertiary-container/10 flex items-center justify-center text-tertiary flex-shrink-0">
                      <span class="icon-[tabler--sparkle]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.32') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.terms.33') }}
                    </p>
                    <div class="grid md:grid-cols-2 gap-4">
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--package] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.terms.34') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.terms.35') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--cash] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.terms.36') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.terms.37') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--users] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.terms.38') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.terms.39') }}</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--chart-bar] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">{{ $t('landing.auto.terms.40') }}</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">{{ $t('landing.auto.terms.41') }}</p>
                      </div>
                    </div>
                    <div class="bg-info/10 border border-info/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--info-circle] text-info mr-2" />
                        {{ $t('landing.auto.terms.42') }}
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 4 -->
                <div id="pagamento" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-success-container/10 flex items-center justify-center text-success flex-shrink-0">
                      <span class="icon-[tabler--credit-card]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.43') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.44') }}</h4>
                      <ul class="space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.45') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.46') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.47') }}</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.48') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.49') }}
                      </p>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.50') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.51') }}
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--alert-triangle] text-error text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.52') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--alert-triangle] text-error text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.53') }}</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                </div>

                <!-- Section 5 -->
                <div id="cancelamento" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-error-container/10 flex items-center justify-center text-error flex-shrink-0">
                      <span class="icon-[tabler--logout]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.54') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.terms.55') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>{{ $t('landing.auto.terms.56') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>{{ $t('landing.auto.terms.57') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>{{ $t('landing.auto.terms.58') }}</span>
                      </li>
                    </ul>
                    <div class="bg-surface p-4 rounded-xl border border-outline-variant/20 mt-4">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--archive] text-on-surface-variant mr-2" />
                        {{ $t('landing.auto.terms.59') }}
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 6 -->
                <div id="propriedade" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary flex-shrink-0">
                      <span class="icon-[tabler--copyright]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.60') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.terms.61') }}
                    </p>
                    <div class="grid md:grid-cols-2 gap-4">
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.62') }}</h4>
                        <ul class="space-y-1 font-body-sm text-body-sm text-on-surface-variant">
                          <li>{{ $t('landing.auto.terms.63') }}</li>
                          <li>{{ $t('landing.auto.terms.64') }}</li>
                          <li>{{ $t('landing.auto.terms.65') }}</li>
                          <li>{{ $t('landing.auto.terms.66') }}</li>
                        </ul>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.67') }}</h4>
                        <ul class="space-y-1 font-body-sm text-body-sm text-on-surface-variant">
                          <li>{{ $t('landing.auto.terms.68') }}</li>
                          <li>{{ $t('landing.auto.terms.69') }}</li>
                          <li>{{ $t('landing.auto.terms.70') }}</li>
                          <li>{{ $t('landing.auto.terms.71') }}</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Section 7 -->
                <div id="limitacao" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-warning-container/10 flex items-center justify-center text-warning flex-shrink-0">
                      <span class="icon-[tabler--report]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.72') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      {{ $t('landing.auto.terms.73') }}
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>{{ $t('landing.auto.terms.74') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>{{ $t('landing.auto.terms.75') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>{{ $t('landing.auto.terms.76') }}</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>{{ $t('landing.auto.terms.77') }}</span>
                      </li>
                    </ul>
                    <div class="bg-warning/10 border border-warning/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <strong class="text-on-surface">{{ $t('landing.auto.terms.78') }}</strong> {{ $t('landing.auto.terms.79') }}
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 8 -->
                <div id="contato" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary flex-shrink-0">
                      <span class="icon-[tabler--mail-opened]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">{{ $t('landing.auto.terms.80') }}</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.81') }}</h4>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <div class="space-y-2">
                          <div class="flex items-center gap-3">
                            <span class="icon-[tabler--mail] text-secondary" />
                            <span class="font-body-md text-body-md text-on-surface">{{ $t('landing.auto.terms.82') }}</span>
                          </div>
                          <div class="flex items-center gap-3">
                            <span class="icon-[tabler--phone] text-secondary" />
                            <span class="font-body-md text-body-md text-on-surface">(11) 99999-9999</span>
                          </div>
                        </div>
                      </div>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.83') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.84') }}
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.85') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.86') }}</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>{{ $t('landing.auto.terms.87') }}</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">{{ $t('landing.auto.terms.88') }}</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        {{ $t('landing.auto.terms.89') }}
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Back to Top -->
              <div class="mt-8 flex justify-center">
                <button 
                  class="btn btn-ghost gap-2"
                  @click="scrollToTop"
                >
                  <span class="icon-[tabler--arrow-up]" />
                  {{ $t('landing.auto.terms.90') }}
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
            {{ $t('landing.auto.terms.91') }}
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <NuxtLink to="/privacy" class="btn btn-outline btn-primary">
              <span class="icon-[tabler--shield] mr-2" />
              {{ $t('landing.auto.terms.92') }}
            </NuxtLink>
            <NuxtLink to="/contact" class="btn btn-ghost">
              <span class="icon-[tabler--headset] mr-2" />
              {{ $t('landing.auto.terms.93') }}
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
