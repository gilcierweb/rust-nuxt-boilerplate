<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="material-symbols-outlined text-sm" data-icon="mail">mail</span>
            Entre em Contato
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            Estamos Aqui para Ajudar
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Tem alguma dúvida? Nossa equipe está pronta para ajudar você a transformar sua lavanderia.
          </p>
        </div>
      </section>

      <!-- Contact Grid -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-3 gap-6 lg:gap-8">
            <!-- Contact Info Cards -->
            <div class="lg:col-span-1 space-y-6">
              <!-- Email -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                  <span class="material-symbols-outlined text-2xl" data-icon="email">email</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">E-mail</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  Resposta em até 24 horas
                </p>
                <a href="mailto:contato@ilaundry.com" class="text-primary font-medium hover:underline">
                  contato@ilaundry.com
                </a>
              </div>

              <!-- Phone -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                  <span class="material-symbols-outlined text-2xl" data-icon="phone">phone</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Telefone</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-3">
                  Seg-Sex, 8h às 18h
                </p>
                <a href="tel:+5511999999999" class="text-secondary font-medium hover:underline">
                  (11) 99999-9999
                </a>
              </div>

              <!-- Address -->
              <div class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <div class="w-12 h-12 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                  <span class="material-symbols-outlined text-2xl" data-icon="location_on">location_on</span>
                </div>
                <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Endereço</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Av. Paulista, 1000<br>
                  Bela Vista, São Paulo - SP<br>
                  CEP: 01310-100
                </p>
              </div>
            </div>

            <!-- Contact Form -->
            <div class="lg:col-span-2">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30">
                <h2 class="font-headline-lg text-headline-lg text-on-surface mb-6">
                  Envie uma Mensagem
                </h2>
                
                <form @submit.prevent="handleSubmit" class="space-y-6">
                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Name -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Nome Completo</span>
                        <span class="text-error">*</span>
                      </label>
                      <input
                        v-model="form.name"
                        type="text"
                        placeholder="Seu nome"
                        class="input input-bordered w-full bg-surface"
                        required
                      />
                    </div>

                    <!-- Email -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">E-mail</span>
                        <span class="text-error">*</span>
                      </label>
                      <input
                        v-model="form.email"
                        type="email"
                        placeholder="seu@email.com"
                        class="input input-bordered w-full bg-surface"
                        required
                      />
                    </div>
                  </div>

                  <div class="grid md:grid-cols-2 gap-6">
                    <!-- Phone -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Telefone</span>
                      </label>
                      <input
                        v-model="form.phone"
                        type="tel"
                        placeholder="(11) 99999-9999"
                        class="input input-bordered w-full bg-surface"
                      />
                    </div>

                    <!-- Subject -->
                    <div class="form-control">
                      <label class="label">
                        <span class="label-text font-medium text-on-surface">Assunto</span>
                        <span class="text-error">*</span>
                      </label>
                      <select v-model="form.subject" class="select select-bordered w-full bg-surface" required>
                        <option value="">Selecione...</option>
                        <option value="sales">Vendas - Quero contratar</option>
                        <option value="support">Suporte Técnico</option>
                        <option value="billing">Faturamento</option>
                        <option value="partnership">Parcerias</option>
                        <option value="other">Outro</option>
                      </select>
                    </div>
                  </div>

                  <!-- Message -->
                  <div class="form-control">
                    <label class="label">
                      <span class="label-text font-medium text-on-surface">Mensagem</span>
                      <span class="text-error">*</span>
                    </label>
                    <textarea
                      v-model="form.message"
                      placeholder="Como podemos ajudar?"
                      class="textarea textarea-bordered w-full bg-surface min-h-[150px]"
                      required
                    ></textarea>
                  </div>

                  <!-- Privacy Checkbox -->
                  <div class="form-control">
                    <label class="label cursor-pointer justify-start gap-3">
                      <input v-model="form.privacy" type="checkbox" class="checkbox checkbox-primary" required />
                      <span class="label-text text-on-surface-variant">
                        Li e concordo com a <NuxtLink to="/privacy" class="text-primary hover:underline">Política de Privacidade</NuxtLink>
                      </span>
                    </label>
                  </div>

                  <!-- Submit -->
                  <div class="flex items-center gap-4 pt-4">
                    <button
                      type="submit"
                      class="btn btn-primary px-8"
                      :disabled="submitting"
                    >
                      <span v-if="submitting" class="loading loading-spinner loading-sm mr-2"></span>
                      <span v-else class="material-symbols-outlined mr-2" data-icon="send">send</span>
                      {{ submitting ? 'Enviando...' : 'Enviar Mensagem' }}
                    </button>
                    
                    <span v-if="success" class="text-success flex items-center gap-2">
                      <span class="material-symbols-outlined" data-icon="check_circle">check_circle</span>
                      Mensagem enviada com sucesso!
                    </span>
                  </div>
                </form>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Perguntas Frequentes</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Respostas rápidas para as dúvidas mais comuns.
            </p>
          </div>

          <div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
            <!-- FAQ 1 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-primary" data-icon="help">help</span>
                Como funciona o teste gratuito?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Oferecemos 14 dias de teste completo sem necessidade de cartão de crédito. Acesso total a todas as funcionalidades.
              </p>
            </div>

            <!-- FAQ 2 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-secondary" data-icon="help">help</span>
                Posso migrar meus dados?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Sim! Nossa equipe de suporte auxilia na migração de dados de qualquer sistema anterior gratuitamente.
              </p>
            </div>

            <!-- FAQ 3 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-tertiary" data-icon="help">help</span>
                Qual o prazo de implementação?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                A maioria dos clientes está operando em até 48 horas. Treinamento incluso e suporte dedicado.
              </p>
            </div>

            <!-- FAQ 4 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="material-symbols-outlined text-success" data-icon="help">help</span>
                Funciona offline?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Sim! O sistema sincroniza automaticamente quando a conexão retorna. Nunca perca uma venda.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Map Placeholder Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="aspect-[21/9] bg-surface-container-low flex items-center justify-center relative">
              <div class="absolute inset-0 bg-gradient-to-br from-primary/5 to-secondary/5"></div>
              <div class="text-center relative z-10">
                <span class="material-symbols-outlined text-6xl text-primary/40 mb-4" data-icon="map">map</span>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Nosso Escritório</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Avenida Paulista, 1000 - São Paulo, SP
                </p>
                <a 
                  href="https://maps.google.com/?q=Av.+Paulista+1000+São+Paulo" 
                  target="_blank"
                  class="btn btn-outline btn-primary mt-6"
                >
                  <span class="material-symbols-outlined mr-2" data-icon="open_in_new">open_in_new</span>
                  Ver no Google Maps
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup>
definePageMeta({
  layout: 'landing'
})

const form = reactive({
  name: '',
  email: '',
  phone: '',
  subject: '',
  message: '',
  privacy: false
})

const submitting = ref(false)
const success = ref(false)

async function handleSubmit() {
  submitting.value = true
  
  // Simulate API call
  await new Promise(resolve => setTimeout(resolve, 1500))
  
  submitting.value = false
  success.value = true
  
  // Reset form
  Object.keys(form).forEach(key => {
    form[key] = key === 'privacy' ? false : ''
  })
  
  // Reset success message after 5 seconds
  setTimeout(() => {
    success.value = false
  }, 5000)
}
</script>