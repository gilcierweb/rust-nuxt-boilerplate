<template>
  <div>
    <header class="fixed top-0 w-full z-50 bg-surface-container-lowest/90 backdrop-blur-md border-b border-outline-variant/50 shadow-sm shadow-primary/5 transition-all duration-300">
      <div class="max-w-[1440px] mx-auto px-6 h-20 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <NuxtLink :to="localePath('/')">
            <span class="icon-[tabler--code] text-primary text-3xl" />
            <span class="text-2xl font-black tracking-tight text-primary font-display-xl">RustNuxt</span>
          </NuxtLink>
        </div>
        <nav class="hidden md:flex items-center gap-8">
          <NuxtLink :to="localePath('/')" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Features</NuxtLink>
          <NuxtLink :to="localePath('/')" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Tech Stack</NuxtLink>
          <NuxtLink :to="localePath('/')" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Architecture</NuxtLink>
          <NuxtLink :to="localePath('/contact')" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Contact</NuxtLink>
        </nav>
        <div class="flex items-center gap-4">
          <LanguageSwitcher />
          <NuxtLink :to="localePath('/auth/login')" class="hidden sm:block text-primary font-semibold font-body-md hover:opacity-80 transition-opacity">Login</NuxtLink>
          <NuxtLink :to="localePath('/auth/register')" class="bg-primary text-on-primary px-5 py-2.5 rounded-lg font-body-md font-medium hover:bg-primary-container hover:text-on-primary-container active:scale-[0.98] transition-all shadow-sm shadow-primary/20">Get Started</NuxtLink>
        </div>
      </div>
    </header>
  </div>
</template>

<script setup lang="ts">
const localePath = useLocalePath()
</script>
