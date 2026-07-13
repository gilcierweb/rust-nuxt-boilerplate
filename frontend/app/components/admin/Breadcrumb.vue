<template>
  <div class="breadcrumbs text-sm">
    <ul>
      <li v-for="(item, index) in items" :key="index">
        <span v-if="index > 0" class="breadcrumbs-separator rtl:rotate-180">
            <span class="icon-[tabler--chevron-right]"></span>
        </span>
        <template v-if="index < items.length - 1">
          <NuxtLink v-if="item.to" :to="item.to">{{ item.label }}</NuxtLink>
          <a v-else-if="item.href" :href="item.href">{{ item.label }}</a>
          <span v-else>{{ item.label }}</span>
        </template>
        <template v-else>
          <span aria-current="page">{{ item.label }}</span>
        </template>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
interface BreadcrumbItem {
  label: string
  to?: string
  href?: string
}

withDefaults(defineProps<{
  items: BreadcrumbItem[]
}>(), {
  items: () => []
})
</script>
