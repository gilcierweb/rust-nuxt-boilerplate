<template>
  <section class="space-y-6">
    <AdminBreadcrumb :items="props.breadcrumbItems" />

    <div :class="isCard ? 'card shadow-base-300/10 shadow-md' : 'rounded-box border border-base-content/10 bg-base-100 p-6 shadow-md shadow-base-content/5'">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div>
          <div class="mb-3 inline-flex items-center gap-2 rounded-field bg-primary/10 px-3 py-1 text-xs font-semibold uppercase tracking-[0.22em] text-primary">
            <span :class="[`icon-[tabler--${props.icon}]`, 'size-4']"></span>
            <span>{{ $t(props.titleKey) }}</span>
          </div>
          <h1 :class="isCard ? 'card-title text-3xl font-semibold tracking-tight text-base-content' : 'text-3xl font-semibold tracking-tight text-base-content'">
            {{ $t(props.titleKey) }}
          </h1>
          <p v-if="props.descriptionKey" class="mt-2 max-w-3xl text-sm leading-relaxed text-base-content/60">
            {{ $t(props.descriptionKey) }}
          </p>
        </div>

        <div :class="isCard ? 'card-actions flex flex-wrap gap-2' : 'flex flex-wrap gap-2'">
          <slot name="header-actions" />
        </div>
      </div>
    </div>

    <slot />
  </section>
</template>

<script setup lang="ts">
// The header of every admin page: breadcrumb, kicker (icon + translated
// title), the description and the page actions. The tables, the forms and
// everything below the header come from the default slot, so the pages keep
// their own wiring (model, sorting, pagination).
//
// `header="card"` swaps the chrome to the card chrome the form pages use;
// the list pages share the same structural layout with the rounded box.
import { computed } from 'vue'

const props = defineProps<{
  /** The tabler icon name of the kicker, e.g. `user` or `shield`. */
  icon: string
  /** The i18n key of the page title, the kicker shows the same key. */
  titleKey: string
  descriptionKey?: string
  breadcrumbItems: Array<{ label: string; to?: string }>
  header?: 'box' | 'card'
}>()

const isCard = computed(() => props.header === 'card')
</script>