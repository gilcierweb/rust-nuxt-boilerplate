<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="material-symbols-outlined text-sm" data-icon="play_circle">play_circle</span>
            Processo Simples
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            Como a iLaundry Funciona
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Em apenas 4 passos simples, transforme sua lavanderia em uma operação moderna e eficiente.
          </p>
        </div>
      </section>

      <!-- Steps Section -->
      <section class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-6">
          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <!-- Step 1 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-primary flex items-center justify-center text-on-primary font-bold text-xl">
                  1
                </div>
                <div class="w-16 h-16 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 mt-4 text-primary">
                  <span class="material-symbols-outlined text-3xl" data-icon="person_add">person_add</span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Cadastro Rápido</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Crie sua conta em menos de 2 minutos. Apenas dados básicos da sua lavanderia.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                    Sem burocracia
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                    14 dias grátis
                  </li>
                </ul>
              </div>
            </div>

            <!-- Step 2 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-secondary flex items-center justify-center text-on-secondary font-bold text-xl">
                  2
                </div>
                <div class="w-16 h-16 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 mt-4 text-secondary">
                  <span class="material-symbols-outlined text-3xl" data-icon="settings">settings</span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Configure</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Personalize serviços, preços, horários e muito mais. Tudo de forma intuitiva.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-secondary text-sm" data-icon="check">check</span>
                    Serviços ilimitados
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-secondary text-sm" data-icon="check">check</span>
                    Tabela de preços
                  </li>
                </ul>
              </div>
            </div>

            <!-- Step 3 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-tertiary flex items-center justify-center text-on-tertiary font-bold text-xl">
                  3
                </div>
                <div class="w-16 h-16 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 mt-4 text-tertiary">
                  <span class="material-symbols-outlined text-3xl" data-icon="shopping_cart">shopping_cart</span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Comece a Vender</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Receba pedidos pelo sistema, app ou presencialmente. Tudo integrado.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-tertiary text-sm" data-icon="check">check</span>
                    PDV integrado
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-tertiary text-sm" data-icon="check">check</span>
                    Controle de caixa
                  </li>
                </ul>
              </div>
            </div>

            <!-- Step 4 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-success flex items-center justify-center text-on-success font-bold text-xl">
                  4
                </div>
                <div class="w-16 h-16 rounded-xl bg-success-container/10 flex items-center justify-center mb-6 mt-4 text-success">
                  <span class="material-symbols-outlined text-3xl" data-icon="trending_up">trending_up</span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Cresça</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Acompanhe relatórios, otimize processos e escale seu negócio.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-success text-sm" data-icon="check">check</span>
                    Relatórios automáticos
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="material-symbols-outlined text-success text-sm" data-icon="check">check</span>
                    Insights de negócio
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Features Deep Dive -->
      <section class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Funcionalidades Principais</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Tudo que você precisa para gerenciar sua lavanderia em um só lugar.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <!-- Feature 1 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="material-symbols-outlined text-3xl" data-icon="point_of_sale">point_of_sale</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">PDV Completo</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Caixa rápido com atalhos, split payments, descontos e múltiplas formas de pagamento.
              </p>
            </div>

            <!-- Feature 2 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="material-symbols-outlined text-3xl" data-icon="inventory_2">inventory_2</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Controle de Estoque</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Gestão de produtos de limpeza, embalagens e insumos com alertas de reposição.
              </p>
            </div>

            <!-- Feature 3 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="material-symbols-outlined text-3xl" data-icon="groups">groups</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Gestão de Clientes</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Cadastro completo, histórico de pedidos, preferências e programa de fidelidade.
              </p>
            </div>

            <!-- Feature 4 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="material-symbols-outlined text-3xl" data-icon="local_shipping">local_shipping</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Delivery & Retirada</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Agendamento de coletas e entregas com rotas otimizadas e notificações automáticas.
              </p>
            </div>

            <!-- Feature 5 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="material-symbols-outlined text-3xl" data-icon="barcode">barcode</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Código de Barras</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Gere etiquetas automaticamente e rastreie cada peça durante todo o processo.
              </p>
            </div>

            <!-- Feature 6 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="material-symbols-outlined text-3xl" data-icon="analytics">analytics</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Relatórios & BI</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Dashboards em tempo real, análise de desempenho e projeções financeiras.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Integration Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Integrações</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Conecte com Suas Ferramentas Favoritas</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
                A iLaundry se integra com as principais plataformas do mercado para uma experiência completa.
              </p>
              
              <div class="space-y-4">
                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="material-symbols-outlined text-2xl" data-icon="payments">payments</span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">Gateways de Pagamento</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Stripe, MercadoPago, PagSeguro, etc.</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-secondary/10 flex items-center justify-center text-secondary">
                    <span class="material-symbols-outlined text-2xl" data-icon="sms">sms</span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">SMS & WhatsApp</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Notificações automáticas aos clientes</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-tertiary/10 flex items-center justify-center text-tertiary">
                    <span class="material-symbols-outlined text-2xl" data-icon="account_balance">account_balance</span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">Contabilidade</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Exportação para sistemas contábeis</p>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-surface p-8 rounded-3xl border border-outline-variant/30 relative overflow-hidden">
              <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
              <div class="relative z-10">
                <div class="aspect-square bg-surface-container-low rounded-2xl flex items-center justify-center">
                  <div class="text-center">
                    <span class="material-symbols-outlined text-8xl text-primary/30 mb-4" data-icon="hub">hub</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">API Aberta</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant max-w-xs">
                      Desenvolvedores podem criar integrações personalizadas
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Pricing Teaser -->
      <section class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Preço Simples e Transparente</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
            Sem taxa de setup. Sem contrato de fidelidade. Cancele quando quiser.
          </p>

          <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 max-w-2xl mx-auto">
            <div class="flex items-center justify-center gap-2 mb-4">
              <span class="font-display-xl text-display-xl text-on-surface">R$ 97</span>
              <span class="font-body-lg text-body-lg text-on-surface-variant">/mês</span>
            </div>
            <p class="font-body-md text-body-md text-on-surface-variant mb-6">
              Acesso ilimitado a todas as funcionalidades. Suporte 24/7 incluso.
            </p>
            <ul class="space-y-3 mb-8 text-left max-w-md mx-auto">
              <li class="flex items-center gap-3 font-body-md text-body-md text-on-surface">
                <span class="material-symbols-outlined text-primary" data-icon="check_circle">check_circle</span>
                Usuários ilimitados
              </li>
              <li class="flex items-center gap-3 font-body-md text-body-md text-on-surface">
                <span class="material-symbols-outlined text-primary" data-icon="check_circle">check_circle</span>
                Clientes ilimitados
              </li>
              <li class="flex items-center gap-3 font-body-md text-body-md text-on-surface">
                <span class="material-symbols-outlined text-primary" data-icon="check_circle">check_circle</span>
                Pedidos ilimitados
              </li>
              <li class="flex items-center gap-3 font-body-md text-body-md text-on-surface">
                <span class="material-symbols-outlined text-primary" data-icon="check_circle">check_circle</span>
                Suporte prioritário
              </li>
            </ul>
            <NuxtLink to="/contact" class="btn btn-primary px-8">
              Começar Teste Grátis
              <span class="material-symbols-outlined ml-2" data-icon="arrow_forward">arrow_forward</span>
            </NuxtLink>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-16 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Pronto para Começar?</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                Junte-se a centenas de lavanderias que já estão crescendo com a iLaundry.
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink to="/contact" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  Fale Conosco <span class="material-symbols-outlined" data-icon="arrow_forward">arrow_forward</span>
                </NuxtLink>
                <NuxtLink to="/about" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  Conheça Nossa História
                </NuxtLink>
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
</script>