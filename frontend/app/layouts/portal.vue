<template>
  <div class="bg-base-200 flex min-h-screen flex-col">
    <PortalNavbar />
    <PortalSidebar />

    <div class="lg:pl-72 flex grow flex-col">
      <main class="mx-auto w-full max-w-[1280px] flex-1 grow space-y-6 p-6">
        <slot />
      </main>
      <PortalFooter />
    </div>
  </div>
</template>

<script setup lang="ts">
</script>
