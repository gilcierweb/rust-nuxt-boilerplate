<template>
  <section class="space-y-6">
    <div class="rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5">
      <h1 class="text-2xl font-semibold text-base-content">{{ $t('portal.dashboard.title') }}</h1>
      <p class="mt-2 text-sm text-base-content/65">
        {{ $t('portal.dashboard.description') }}
      </p>
    </div>

    <div class="rounded-box border border-base-content/10 bg-base-100 p-5">
      <h2 class="text-lg font-semibold text-base-content mb-4">{{ $t('portal.dashboard.quickAccess') }}</h2>
      <div class="grid gap-4 sm:grid-cols-2">
        <a href="#" class="btn btn-outline btn-wide">
          {{ $t('portal.dashboard.cards.documents') }}
        </a>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'portal' })
</script>
