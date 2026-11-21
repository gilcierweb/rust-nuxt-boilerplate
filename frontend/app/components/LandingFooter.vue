<template>
  <footer class="bg-surface-container-low py-12 px-6 border-t border-outline-variant/20">
    <div class="max-w-[1440px] mx-auto grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-8 mb-12">
      <div class="col-span-2">
        <div class="flex items-center gap-2 mb-4">
          <span class="icon-[tabler--code] text-primary text-3xl" />
          <span class="text-xl font-bold text-on-surface font-display-xl tracking-tight">{{ $t('landing.auto.LandingFooter.1') }}</span>
        </div>
        <p class="text-sm leading-relaxed text-on-surface-variant mb-6 max-w-xs">{{ $t('landing.auto.LandingFooter.2') }}</p>
        <div class="flex gap-4">
          <a class="w-8 h-8 rounded-full bg-surface-container flex items-center justify-center text-on-surface-variant hover:text-primary transition-colors" href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank"><span class="icon-[tabler--code] text-sm" /></a>
          <a class="w-8 h-8 rounded-full bg-surface-container flex items-center justify-center text-on-surface-variant hover:text-primary transition-colors" href="mailto:contact@example.com"><span class="icon-[tabler--mail] text-sm" /></a>
        </div>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">{{ $t('landing.auto.LandingFooter.3') }}</h4>
        <ul class="space-y-3">
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#features">{{ $t('landing.auto.LandingFooter.4') }}</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#tech-stack">{{ $t('landing.auto.LandingFooter.5') }}</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#architecture">{{ $t('landing.auto.LandingFooter.6') }}</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#security">{{ $t('landing.auto.LandingFooter.7') }}</a></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">{{ $t('landing.auto.LandingFooter.8') }}</h4>
        <ul class="space-y-3">
          <li><NuxtLink :to="localePath('/about')" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">{{ $t('landing.auto.LandingFooter.9') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/how-work')" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">{{ $t('landing.auto.LandingFooter.10') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/contact')" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">{{ $t('landing.auto.LandingFooter.11') }}</NuxtLink></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">{{ $t('landing.auto.LandingFooter.12') }}</h4>
        <ul class="space-y-3">
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank">{{ $t('landing.auto.LandingFooter.13') }}</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/swagger-ui/">{{ $t('landing.auto.LandingFooter.14') }}</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#getting-started">{{ $t('landing.auto.LandingFooter.15') }}</a></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">{{ $t('landing.auto.LandingFooter.16') }}</h4>
        <ul class="space-y-3">
          <li><NuxtLink :to="localePath('/privacy')" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">{{ $t('landing.auto.LandingFooter.17') }}</NuxtLink></li>
          <li><NuxtLink :to="localePath('/terms')" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">{{ $t('landing.auto.LandingFooter.18') }}</NuxtLink></li>
        </ul>
      </div>
    </div>
    <div class="max-w-[1440px] mx-auto pt-8 border-t border-outline-variant/20 text-center md:text-left">
      <p class="text-sm leading-relaxed text-on-surface-variant">&copy; {{ new Date().getFullYear() }} {{ $t('landing.footer.copyright') }}</p>
    </div>
  </footer>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
