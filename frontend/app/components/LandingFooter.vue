<template>
  <footer class="bg-surface-container-low py-12 px-6 border-t border-outline-variant/20">
    <div class="max-w-[1440px] mx-auto grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-8 mb-12">
      <div class="col-span-2">
        <div class="flex items-center gap-2 mb-4">
          <span class="material-symbols-outlined text-primary text-3xl" data-icon="code">code</span>
          <span class="text-xl font-bold text-on-surface font-display-xl tracking-tight">RustNuxt Boilerplate</span>
        </div>
        <p class="text-sm leading-relaxed text-on-surface-variant mb-6 max-w-xs">A production-ready Rust (Actix Web) + Nuxt 4 boilerplate with authentication, RBAC, admin panel, and modern developer experience.</p>
        <div class="flex gap-4">
          <a class="w-8 h-8 rounded-full bg-surface-container flex items-center justify-center text-on-surface-variant hover:text-primary transition-colors" href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank"><span class="material-symbols-outlined text-sm" data-icon="code">code</span></a>
          <a class="w-8 h-8 rounded-full bg-surface-container flex items-center justify-center text-on-surface-variant hover:text-primary transition-colors" href="mailto:contact@example.com"><span class="material-symbols-outlined text-sm" data-icon="mail">mail</span></a>
        </div>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">Product</h4>
        <ul class="space-y-3">
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#features">Features</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#tech-stack">Tech Stack</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#architecture">Architecture</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#security">Security</a></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">Company</h4>
        <ul class="space-y-3">
          <li><NuxtLink to="/about" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">About</NuxtLink></li>
          <li><NuxtLink to="/how-work" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">How It Works</NuxtLink></li>
          <li><NuxtLink to="/contact" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">Contact</NuxtLink></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">Resources</h4>
        <ul class="space-y-3">
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank">GitHub</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/swagger-ui/">API Docs</a></li>
          <li><a class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none" href="/#getting-started">Quick Start</a></li>
        </ul>
      </div>
      <div>
        <h4 class="font-bold text-on-surface mb-4 text-sm uppercase tracking-wider">Legal</h4>
        <ul class="space-y-3">
          <li><NuxtLink to="/privacy" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">Privacy Policy</NuxtLink></li>
          <li><NuxtLink to="/terms" class="text-sm leading-relaxed text-on-surface-variant hover:text-primary hover:underline transition-all focus:ring-2 focus:ring-primary outline-none">Terms of Service</NuxtLink></li>
        </ul>
      </div>
    </div>
    <div class="max-w-[1440px] mx-auto pt-8 border-t border-outline-variant/20 text-center md:text-left">
      <p class="text-sm leading-relaxed text-on-surface-variant">&copy; {{ new Date().getFullYear() }} RustNuxt Boilerplate. MIT Licensed. All rights reserved.</p>
    </div>
  </footer>
</template>
