<template>
  <div class="min-h-screen flex flex-col bg-background text-on-background font-body-md antialiased overflow-x-hidden">
    <LandingNavbar />
    <main class="flex-1">
      <slot />
    </main>
    <LandingFooter />
  </div>
</template>

<script setup lang="ts">
useHead({
  link: [
    {
      rel: 'stylesheet',
      href: 'https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;900&display=swap'
    }
  ]
})

import '~/assets/css/landing.css'
</script>