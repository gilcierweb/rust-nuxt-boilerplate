<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-16 lg:py-24">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-6 relative z-10">
          <div class="max-w-3xl">
            <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
              <span class="icon-[tabler--scale] text-sm" />
              Acordo de Uso
            </div>
            <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight">
              Termos de Serviço
            </h1>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
              Última atualização: 3 de Maio de 2026
            </p>
          </div>
        </div>
      </section>

      <!-- Content Section -->
      <section class="py-12 md:py-20 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-4 gap-6 lg:gap-12">
            <!-- Sidebar Navigation -->
            <div class="hidden lg:block lg:col-span-1">
              <div class="sticky top-24 bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30">
                <h3 class="font-headline-sm text-headline-sm text-on-surface mb-4">Índice</h3>
                <nav class="space-y-2">
                  <a href="#aceitacao" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    1. Aceitação
                  </a>
                  <a href="#conta" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    2. Sua Conta
                  </a>
                  <a href="#servicos" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    3. Nossos Serviços
                  </a>
                  <a href="#pagamento" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    4. Pagamento
                  </a>
                  <a href="#cancelamento" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    5. Cancelamento
                  </a>
                  <a href="#propriedade" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    6. Propriedade
                  </a>
                  <a href="#limitacao" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    7. Limitação
                  </a>
                  <a href="#contato" class="block py-2 px-3 rounded-lg font-body-md text-body-md text-on-surface-variant hover:bg-surface-container hover:text-on-surface transition-colors">
                    8. Contato
                  </a>
                </nav>
              </div>
            </div>

            <!-- Main Content -->
            <div class="lg:col-span-3">
              <div class="bg-surface-container-lowest p-5 md:p-8 lg:p-10 rounded-2xl border border-outline-variant/30 space-y-8 md:space-y-12">
                <!-- Intro -->
                <div class="prose prose-lg max-w-none">
                  <p class="font-body-lg text-body-lg text-on-surface-variant leading-relaxed">
                    Estes Termos de Serviço regem o uso da plataforma iLaundry. Ao acessar ou usar nossos serviços, você concorda em cumprir estes termos. Leia atentamente antes de continuar.
                  </p>
                </div>

                <!-- Section 1 -->
                <div id="aceitacao" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary flex-shrink-0">
                      <span class="icon-[tabler--circle-check]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">1. Aceitação dos Termos</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      Ao criar uma conta ou usar a iLaundry, você concorda com:
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>Estes Termos de Serviço na íntegra</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>Nossa <NuxtLink to="/privacy" class="text-primary hover:underline">Política de Privacidade</NuxtLink></span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-primary mt-0.5" />
                        <span>Quaisquer diretrizes ou regras adicionais publicadas na plataforma</span>
                      </li>
                    </ul>
                    <div class="bg-warning/10 border border-warning/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--alert-triangle] text-warning mr-2" />
                        Se você não concordar com estes termos, não utilize nossos serviços.
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 2 -->
                <div id="conta" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary flex-shrink-0">
                      <span class="icon-[tabler--user]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">2. Sua Conta</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">2.1 Cadastro</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Para usar a iLaundry, você deve:
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Ser maior de 18 anos ou ter autorização do responsável legal</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Fornecer informações verdadeiras, atualizadas e completas</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Ter capacidade legal para celebrar contratos</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">2.2 Segurança</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Você é responsável por:
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Manter sua senha em sigilo absoluto</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Notificar imediatamente qualquer uso não autorizado</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Garantir a segurança dos dispositivos usados para acesso</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                </div>

                <!-- Section 3 -->
                <div id="servicos" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-tertiary-container/10 flex items-center justify-center text-tertiary flex-shrink-0">
                      <span class="icon-[tabler--sparkle]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">3. Nossos Serviços</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      A iLaundry oferece:
                    </p>
                    <div class="grid md:grid-cols-2 gap-4">
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--package] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">Gestão de Pedidos</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">Controle completo do fluxo de trabalho</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--cash] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">PDV Integrado</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">Sistema de caixa completo</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--users] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">CRM de Clientes</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">Gestão de relacionamento</p>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <span class="icon-[tabler--chart-bar] text-tertiary mb-2" />
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-1">Relatórios</h4>
                        <p class="font-body-sm text-body-sm text-on-surface-variant">Análises e dashboards</p>
                      </div>
                    </div>
                    <div class="bg-info/10 border border-info/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--info-circle] text-info mr-2" />
                        Reservamo-nos o direito de modificar, suspender ou descontinuar qualquer serviço mediante aviso prévio de 30 dias.
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 4 -->
                <div id="pagamento" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-success-container/10 flex items-center justify-center text-success flex-shrink-0">
                      <span class="icon-[tabler--credit-card]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">4. Pagamento e Faturamento</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">4.1 Preços</h4>
                      <ul class="space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>Os preços são disponibilizados em reais brasileiros (BRL)</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>Cobrança mensal recorrente via cartão de crédito ou boleto</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-success text-sm mt-1" />
                          <span>Reajustes anuais conforme índice de mercado (IPC-A)</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">4.2 Teste Gratuito</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Oferecemos 14 dias de teste gratuito. Após este período, será necessário escolher um plano pago para continuar usando a plataforma.
                      </p>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">4.3 Inadimplência</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Em caso de atraso no pagamento:
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--alert-triangle] text-error text-sm mt-1" />
                          <span>Suspensão do acesso após 5 dias de atraso</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--alert-triangle] text-error text-sm mt-1" />
                          <span>Exclusão permanente dos dados após 90 dias sem pagamento</span>
                        </li>
                      </ul>
                    </div>
                  </div>
                </div>

                <!-- Section 5 -->
                <div id="cancelamento" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-error-container/10 flex items-center justify-center text-error flex-shrink-0">
                      <span class="icon-[tabler--logout]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">5. Cancelamento</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      Você pode cancelar sua assinatura a qualquer momento:
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>Acesso mantido até o final do período pago</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>Exportação de dados disponível por 30 dias após cancelamento</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--check] text-error mt-0.5" />
                        <span>Reembolso pro-rata disponível em caso de cancelamento no primeiro mês</span>
                      </li>
                    </ul>
                    <div class="bg-surface p-4 rounded-xl border border-outline-variant/20 mt-4">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <span class="icon-[tabler--archive] text-on-surface-variant mr-2" />
                        Recomendamos exportar seus dados antes de cancelar.
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 6 -->
                <div id="propriedade" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-primary-container/10 flex items-center justify-center text-primary flex-shrink-0">
                      <span class="icon-[tabler--copyright]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">6. Propriedade Intelectual</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      Todos os direitos sobre a iLaundry são reservados:
                    </p>
                    <div class="grid md:grid-cols-2 gap-4">
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">Nossa Propriedade</h4>
                        <ul class="space-y-1 font-body-sm text-body-sm text-on-surface-variant">
                          <li>• Software e código-fonte</li>
                          <li>• Logotipos e marcas registradas</li>
                          <li>• Design e interfaces</li>
                          <li>• Documentação e conteúdo</li>
                        </ul>
                      </div>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">Sua Propriedade</h4>
                        <ul class="space-y-1 font-body-sm text-body-sm text-on-surface-variant">
                          <li>• Dados dos seus clientes</li>
                          <li>• Pedidos e transações</li>
                          <li>• Configurações personalizadas</li>
                          <li>• Histórico de operações</li>
                        </ul>
                      </div>
                    </div>
                  </div>
                </div>

                <!-- Section 7 -->
                <div id="limitacao" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-warning-container/10 flex items-center justify-center text-warning flex-shrink-0">
                      <span class="icon-[tabler--report]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">7. Limitação de Responsabilidade</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-4">
                    <p class="font-body-md text-body-md text-on-surface-variant">
                      A iLaundry não se responsabiliza por:
                    </p>
                    <ul class="space-y-3 font-body-md text-body-md text-on-surface-variant">
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>Perdas de dados causadas por falhas do usuário em fazer backups</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>Interrupções de serviço por motivos de força maior (guerra, desastres naturais)</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>Danos indiretos, incidentais ou consequenciais</span>
                      </li>
                      <li class="flex items-start gap-3">
                        <span class="icon-[tabler--arrow-right] text-warning mt-0.5" />
                        <span>Uso indevido da plataforma por terceiros com acesso à sua conta</span>
                      </li>
                    </ul>
                    <div class="bg-warning/10 border border-warning/20 p-4 rounded-xl">
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        <strong class="text-on-surface">Responsabilidade máxima:</strong> Limitada ao valor total pago pelo usuário nos últimos 12 meses.
                      </p>
                    </div>
                  </div>
                </div>

                <!-- Section 8 -->
                <div id="contato" class="scroll-mt-24">
                  <div class="flex items-start md:items-center gap-3 mb-4">
                    <div class="w-10 h-10 rounded-lg bg-secondary-container/10 flex items-center justify-center text-secondary flex-shrink-0">
                      <span class="icon-[tabler--mail-opened]" />
                    </div>
                    <h2 class="font-headline-lg text-headline-lg text-on-surface">8. Contato e Disputas</h2>
                  </div>
                  <div class="pl-0 md:pl-14 space-y-6">
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">8.1 Contato</h4>
                      <div class="bg-surface p-4 rounded-xl border border-outline-variant/20">
                        <div class="space-y-2">
                          <div class="flex items-center gap-3">
                            <span class="icon-[tabler--mail] text-secondary" />
                            <span class="font-body-md text-body-md text-on-surface">juridico@ilaundry.com</span>
                          </div>
                          <div class="flex items-center gap-3">
                            <span class="icon-[tabler--phone] text-secondary" />
                            <span class="font-body-md text-body-md text-on-surface">(11) 99999-9999</span>
                          </div>
                        </div>
                      </div>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">8.2 Resolução de Disputas</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Qualquer disputa será resolvida:
                      </p>
                      <ul class="mt-2 space-y-2 font-body-md text-body-md text-on-surface-variant">
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Primeiramente por negociação amigável entre as partes</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Mediante mediação, se necessário</span>
                        </li>
                        <li class="flex items-start gap-2">
                          <span class="icon-[tabler--check] text-secondary text-sm mt-1" />
                          <span>Foro: Comarca de São Paulo, SP</span>
                        </li>
                      </ul>
                    </div>
                    <div>
                      <h4 class="font-headline-sm text-headline-sm text-on-surface mb-2">8.3 Alterações</h4>
                      <p class="font-body-md text-body-md text-on-surface-variant">
                        Podemos atualizar estes termos a qualquer momento. Alterações significativas serão notificadas por e-mail com 30 dias de antecedência.
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              <!-- Back to Top -->
              <div class="mt-8 flex justify-center">
                <button 
                  @click="scrollToTop"
                  class="btn btn-ghost gap-2"
                >
                  <span class="icon-[tabler--arrow-up]" />
                  Voltar ao topo
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-16 bg-surface-container-lowest border-t border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-6 text-center">
          <p class="font-body-md text-body-md text-on-surface-variant mb-4">
            Ao usar a iLaundry, você confirma que leu, entendeu e concorda com estes Termos de Serviço.
          </p>
          <div class="flex flex-wrap justify-center gap-4">
            <NuxtLink to="/privacy" class="btn btn-outline btn-primary">
              <span class="icon-[tabler--shield] mr-2" />
              Política de Privacidade
            </NuxtLink>
            <NuxtLink to="/contact" class="btn btn-ghost">
              <span class="icon-[tabler--headset] mr-2" />
              Falar com Suporte
            </NuxtLink>
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

function scrollToTop() {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}
</script>
