<template>
  <section class="space-y-6">
    <div class="rounded-box border border-base-content/10 bg-base-100 p-6">
      <h1 class="text-2xl font-semibold text-base-content">{{ $t('portal.support.title') }}</h1>
      <p class="mt-2 text-sm text-base-content/65">
        Para atendimento rápido, entre em contato via WhatsApp.
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
          Abrir WhatsApp
        </a>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
definePageMeta({ layout: 'portal' })

const message = ref('')
const whatsappBase = 'https://wa.me/'

const whatsappHref = computed(() => {
  const text = message.value.trim() || 'Olá, preciso de suporte no portal do cliente.'
  return `${whatsappBase}?text=${encodeURIComponent(text)}`
})
</script>
