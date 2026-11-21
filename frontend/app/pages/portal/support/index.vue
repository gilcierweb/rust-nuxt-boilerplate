<template>
  <section class="space-y-6">
    <div class="rounded-box border border-base-content/10 bg-base-100 p-6">
      <h1 class="text-2xl font-semibold text-base-content">{{ $t('portal.support.title') }}</h1>
      <p class="mt-2 text-sm text-base-content/65">
        {{ $t('portal.support.quickContact') }}
      </p>
    </div>

    <div class="rounded-box border border-base-content/10 bg-base-100 p-6">
      <label class="label-text font-semibold" for="supportMessage">{{ $t('portal.support.messageLabel') }}</label>
      <textarea
        id="supportMessage"
        v-model="message"
        class="textarea mt-2 w-full"
        rows="5"
        :placeholder="$t('portal.support.messagePlaceholder')"
      ></textarea>

      <div class="mt-4 flex flex-wrap justify-end gap-2">
        <a :href="whatsappHref" target="_blank" rel="noopener noreferrer" class="btn btn-primary">
          <span class="icon-[tabler--brand-whatsapp] size-5"></span>
          {{ $t('portal.support.openWhatsApp') }}
        </a>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'PortalSupportIndexPage' })

definePageMeta({ layout: 'portal' })

const message = ref('')
const whatsappBase = 'https://wa.me/'

const whatsappHref = computed(() => {
  const text = message.value.trim() || $t('portal.support.defaultWhatsAppText')
  return `${whatsappBase}?text=${encodeURIComponent(text)}`
})
</script>
