<template>
	<div>
    <!-- Section 1: Navigation -->
    <header class="fixed top-0 w-full z-50 bg-white/90 backdrop-blur-md dark:bg-slate-900/90 border-b border-slate-200/50 dark:border-slate-800 shadow-sm shadow-blue-500/5 transition-all duration-300">
      <div class="max-w-[1440px] mx-auto px-6 h-20 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <NuxtLink to="/">
            <span class="material-symbols-outlined text-[#2F80ED] text-3xl" data-icon="local_laundry_service">local_laundry_service</span>
            <span class="text-2xl font-black tracking-tight text-[#2F80ED] font-display-xl">iLaundry</span>
          </NuxtLink>
        </div>
        <nav class="hidden md:flex items-center gap-8">
          <NuxtLink to="/#features" class="text-slate-600 dark:text-slate-400 font-medium hover:text-[#2F80ED] transition-colors font-body-md">Features</NuxtLink>
          <NuxtLink to="/#roles" class="text-slate-600 dark:text-slate-400 font-medium hover:text-[#2F80ED] transition-colors font-body-md">Roles</NuxtLink>
          <NuxtLink to="/#pricing" class="text-slate-600 dark:text-slate-400 font-medium hover:text-[#2F80ED] transition-colors font-body-md">Pricing</NuxtLink>
          <NuxtLink to="/contact" class="text-slate-600 dark:text-slate-400 font-medium hover:text-[#2F80ED] transition-colors font-body-md">Contact</NuxtLink>
        </nav>
        <div class="flex items-center gap-4">
          <NuxtLink to="/auth/login"  class="hidden sm:block text-[#2F80ED] font-semibold font-body-md hover:opacity-80 transition-opacity">Login</NuxtLink>
          <NuxtLink to="/auth/register" class="bg-[#2F80ED] text-white px-5 py-2.5 rounded-lg font-body-md font-medium hover:bg-[#2563EB] active:scale-[0.98] transition-all shadow-sm">Get Started</NuxtLink>
        </div>
      </div>
    </header>
	</div>
</template>
