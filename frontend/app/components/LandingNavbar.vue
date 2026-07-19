<template>
  <div>
    <header class="fixed top-0 w-full z-50 bg-surface-container-lowest/90 backdrop-blur-md border-b border-outline-variant/50 shadow-sm shadow-primary/5 transition-all duration-300">
      <div class="max-w-[1440px] mx-auto px-6 h-20 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <NuxtLink to="/">
            <span class="icon-[tabler--code] text-primary text-3xl" />
            <span class="text-2xl font-black tracking-tight text-primary font-display-xl">RustNuxt</span>
          </NuxtLink>
        </div>
        <nav class="hidden md:flex items-center gap-8">
          <NuxtLink to="/#features" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Features</NuxtLink>
          <NuxtLink to="/#tech-stack" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Tech Stack</NuxtLink>
          <NuxtLink to="/#architecture" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Architecture</NuxtLink>
          <NuxtLink to="/contact" class="text-on-surface-variant font-medium hover:text-primary transition-colors font-body-md">Contact</NuxtLink>
        </nav>
        <div class="flex items-center gap-4">
          <LanguageSwitcher />
          <NuxtLink to="/auth/login" class="hidden sm:block text-primary font-semibold font-body-md hover:opacity-80 transition-opacity">Login</NuxtLink>
          <NuxtLink to="/auth/register" class="bg-primary text-on-primary px-5 py-2.5 rounded-lg font-body-md font-medium hover:bg-primary-container hover:text-on-primary-container active:scale-[0.98] transition-all shadow-sm shadow-primary/20">Get Started</NuxtLink>
        </div>
      </div>
    </header>
  </div>
</template>
