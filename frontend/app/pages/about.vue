<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="material-symbols-outlined text-sm" data-icon="info">info</span>
            Nossa História
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            Revolucionando a Gestão de Lavanderias
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Desde 2020, transformamos operações de lavanderia com tecnologia intuitiva e design focado na eficiência.
          </p>
        </div>
      </section>

      <!-- Missão, Visão, Valores -->
      <section class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid md:grid-cols-3 gap-6 lg:gap-8">
            <!-- Missão -->
            <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="material-symbols-outlined text-3xl" data-icon="rocket_launch">rocket_launch</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Missão</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Simplificar a gestão de lavanderias através de tecnologia acessível, permitindo que empresas foquem no que fazem de melhor: entregar limpeza e cuidado.
              </p>
            </div>

            <!-- Visão -->
            <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="material-symbols-outlined text-3xl" data-icon="visibility">visibility</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Visão</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Ser a plataforma líder mundial em gestão de lavanderias, reconhecida pela inovação, confiabilidade e impacto positivo nos negócios dos nossos clientes.
              </p>
            </div>

            <!-- Valores -->
            <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="material-symbols-outlined text-3xl" data-icon="verified">verified</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Valores</h3>
              <ul class="space-y-2 font-body-md text-body-md text-on-surface-variant">
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Eficiência em primeiro lugar
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Transparência total
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Inovação contínua
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Foco no cliente
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      <!-- Timeline / Nossa Jornada -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Nossa Jornada</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              De startup visionária a líder de mercado em poucos anos.
            </p>
          </div>

          <div class="relative">
            <!-- Timeline Line -->
            <div class="absolute left-1/2 transform -translate-x-1/2 h-full w-0.5 bg-outline-variant/50 hidden md:block"></div>

            <div class="space-y-12">
              <!-- 2020 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">2020</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">O Início</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Fundação da iLaundry com a missão de digitalizar pequenas lavanderias.
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2021 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-secondary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-secondary tracking-widest uppercase">2021</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Primeiros 100 Clientes</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Conquista dos primeiros 100 clientes ativos e expansão para 3 estados.
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2022 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-tertiary tracking-widest uppercase">2022</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Lançamento Mobile</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      App nativo para iOS e Android, permitindo gestão completa na palma da mão.
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-tertiary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2023 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">2023</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Expansão Nacional</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Presença em todos os estados do Brasil e primeiros clientes internacionais.
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2024 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-success tracking-widest uppercase">2024</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Líder de Mercado</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      500+ estabelecimentos ativos processando 2M+ de itens mensalmente.
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-success border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Team Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Nosso Time</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Profissionais apaixonados por tecnologia e serviço de excelência.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <!-- Member 1 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="person">person</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Ana Silva</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">CEO & Fundadora</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                15 anos de experiência em tecnologia e operações.
              </p>
            </div>

            <!-- Member 2 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-secondary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-secondary" data-icon="person">person</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Carlos Mendes</h3>
              <p class="font-body-sm text-body-sm text-secondary mt-1">CTO</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Especialista em arquitetura de sistemas escaláveis.
              </p>
            </div>

            <!-- Member 3 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-tertiary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-tertiary" data-icon="person">person</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Marina Costa</h3>
              <p class="font-body-sm text-body-sm text-tertiary mt-1">Head de Produto</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Focada em experiência do usuário e design.
              </p>
            </div>

            <!-- Member 4 -->
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="person">person</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Pedro Lima</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">Head de Sucesso</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Garantindo que cada cliente alcance resultados.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Stats Section -->
      <section class="py-16 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid grid-cols-2 md:grid-cols-4 gap-6 lg:gap-8">
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-primary mb-2">500+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Clientes Ativos</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-secondary mb-2">50+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Funcionários</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-tertiary mb-2">27</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Estados Atendidos</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-success mb-2">99.9%</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Uptime</div>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Faça Parte da Nossa História</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                Junte-se a centenas de lavanderias que já transformaram seus negócios com a iLaundry.
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink to="/contact" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  Fale Conosco <span class="material-symbols-outlined" data-icon="arrow_forward">arrow_forward</span>
                </NuxtLink>
                <NuxtLink to="/how_work" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  Como Funciona
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