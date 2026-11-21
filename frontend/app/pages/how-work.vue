<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="icon-[tabler--player-play] text-sm"></span>
            { $t('landing.auto.how_work.1') }
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            { $t('landing.auto.how_work.2') }
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            { $t('landing.auto.how_work.3') }
          </p>
        </div>
      </section>

      <!-- Steps Section -->
      <section id="steps" class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-6">
          <div class="grid md:grid-cols-2 lg:grid-cols-5 gap-8">
            <!-- Step 1 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-primary flex items-center justify-center text-on-primary font-bold text-xl">
                  1
                </div>
                <div class="w-16 h-16 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 mt-4 text-primary">
                  <span class="icon-[tabler--download] text-3xl"></span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.how_work.4') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  { $t('landing.auto.how_work.5') }
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.6') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.7') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.8') }
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
                  <span class="icon-[tabler--adjustments] text-3xl"></span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.how_work.9') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  { $t('landing.auto.how_work.10') }
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    { $t('landing.auto.how_work.11') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    { $t('landing.auto.how_work.12') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    { $t('landing.auto.how_work.13') }
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
                  <span class="icon-[tabler--user-plus] text-3xl"></span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.how_work.14') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  { $t('landing.auto.how_work.15') }
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    { $t('landing.auto.how_work.16') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    { $t('landing.auto.how_work.17') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    { $t('landing.auto.how_work.18') }
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
                  <span class="icon-[tabler--shield-cog] text-3xl"></span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.how_work.19') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  { $t('landing.auto.how_work.20') }
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    { $t('landing.auto.how_work.21') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    { $t('landing.auto.how_work.22') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    { $t('landing.auto.how_work.23') }
                  </li>
                </ul>
              </div>
            </div>

            <!-- Step 5 -->
            <div class="relative">
              <div class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 h-full hover:shadow-lg transition-all duration-300">
                <div class="absolute -top-4 -left-4 w-12 h-12 rounded-full bg-primary flex items-center justify-center text-on-primary font-bold text-xl">
                  5
                </div>
                <div class="w-16 h-16 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 mt-4 text-primary">
                  <span class="icon-[tabler--rocket] text-3xl"></span>
                </div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.how_work.24') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  { $t('landing.auto.how_work.25') }
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.26') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.27') }
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    { $t('landing.auto.how_work.28') }
                  </li>
                </ul>
</div>
            </div>
          </div>
        </div>
      </section>

      <!-- Deep Dive Features -->
      <section id="features" class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{ $t('landing.auto.how_work.29') }</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              { $t('landing.auto.how_work.30') }
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--shield] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.31') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.32') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--shield-cog] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.33') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.34') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--database] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.35') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.36') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--api] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.37') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.38') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--chart-bar] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.39') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.40') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--file-code-2] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.41') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.42') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--shield-lock] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.43') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.44') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--cloud] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.45') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.46') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--puzzle] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">{ $t('landing.auto.how_work.47') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.48') }
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Architecture Overview -->
      <section id="architecture" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">{ $t('landing.auto.how_work.49') }</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">{ $t('landing.auto.how_work.50') }</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
                { $t('landing.auto.how_work.51') }
              </p>
              <div class="space-y-6">
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--building]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.how_work.52') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.how_work.53') }</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--server]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.how_work.54') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.how_work.55') }</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--tool]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.how_work.56') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.how_work.57') }</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--cloud]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.how_work.58') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.how_work.59') }</p>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-surface p-8 rounded-3xl border border-outline-variant/30 relative overflow-hidden">
              <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
              <div class="relative z-10">
                <div class="aspect-square bg-surface-container-low rounded-2xl flex items-center justify-center p-8">
                  <div class="text-center">
                    <svg viewBox="0 0 400 400" class="w-full h-full max-w-full" style="max-width: 400px; max-height: 400px;">
                      <!-- Domain Layer (center) -->
                      <circle cx="200" cy="200" r="60" fill="#E3F2FD" stroke="#1976D2" stroke-width="2" />
                      <text x="200" y="205" text-anchor="middle" font-size="14" fill="#1976D2" font-weight="bold" font-family="system-ui">{ $t('landing.auto.how_work.60') }</text>
                      <text x="200" y="220" text-anchor="middle" font-size="11" fill="#1976D2" font-family="system-ui">{ $t('landing.auto.how_work.61') }</text>
                      
                      <!-- Data Layer -->
                      <ellipse cx="200" cy="200" rx="110" ry="90" fill="none" stroke="#388E3C" stroke-width="2" stroke-dasharray="5,5" />
                      <text x="200" y="110" text-anchor="middle" font-size="12" fill="#388E3C" font-weight="bold" font-family="system-ui">{ $t('landing.auto.how_work.62') }</text>
                      <text x="200" y="125" text-anchor="middle" font-size="10" fill="#388E3C" font-family="system-ui">{ $t('landing.auto.how_work.63') }</text>
                      
                      <!-- Application Layer -->
                      <ellipse cx="200" cy="200" rx="160" ry="130" fill="none" stroke="#F57C00" stroke-width="2" stroke-dasharray="5,5" />
                      <text x="200" y="320" text-anchor="middle" font-size="12" fill="#F57C00" font-weight="bold" font-family="system-ui">{ $t('landing.auto.how_work.64') }</text>
                      <text x="200" y="335" text-anchor="middle" font-size="10" fill="#F57C00" font-family="system-ui">{ $t('landing.auto.how_work.65') }</text>
                      
                      <!-- Interface Layer -->
                      <ellipse cx="200" cy="200" rx="200" ry="165" fill="none" stroke="#7B1FA2" stroke-width="2" stroke-dasharray="5,5" />
                      <text x="380" y="205" text-anchor="middle" font-size="12" fill="#7B1FA2" font-weight="bold" font-family="system-ui">{ $t('landing.auto.how_work.66') }</text>
                      <text x="380" y="220" text-anchor="middle" font-size="10" fill="#7B1FA2" font-family="system-ui">{ $t('landing.auto.how_work.67') }</text>
                      
                      <!-- Arrows showing dependency direction -->
                      <path d="M260 200 L300 200" stroke="#7B1FA2" stroke-width="2" marker-end="url(#arrowhead)" />
                      <path d="M140 200 L100 200" stroke="#7B1FA2" stroke-width="2" marker-end="url(#arrowhead)" />
                      <path d="M200 140 L200 100" stroke="#388E3C" stroke-width="2" marker-end="url(#arrowhead)" />
                      <path d="M200 260 L200 300" stroke="#F57C00" stroke-width="2" marker-end="url(#arrowhead)" />
                      
                      <defs>
                        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                          <polygon points="0 0, 10 3.5, 0 7" fill="#7B1FA2" />
                        </marker>
                      </defs>
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Integration Section -->
      <section id="integrations" class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">{ $t('landing.auto.how_work.68') }</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">{ $t('landing.auto.how_work.69') }</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
                { $t('landing.auto.how_work.70') }
              </p>
              
              <div class="space-y-4">
                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="icon-[tabler--credit-card] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">{ $t('landing.auto.how_work.71') }</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">{ $t('landing.auto.how_work.72') }</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-secondary/10 flex items-center justify-center text-secondary">
                    <span class="icon-[tabler--message] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">{ $t('landing.auto.how_work.73') }</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">{ $t('landing.auto.how_work.74') }</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-tertiary/10 flex items-center justify-center text-tertiary">
                    <span class="icon-[tabler--building-bank] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">{ $t('landing.auto.how_work.75') }</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">{ $t('landing.auto.how_work.76') }</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="icon-[tabler--cloud] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">{ $t('landing.auto.how_work.77') }</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">{ $t('landing.auto.how_work.78') }</p>
                  </div>
                </div>
              </div>
            </div>

            <div class="bg-surface p-8 rounded-3xl border border-outline-variant/30 relative overflow-hidden">
              <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
              <div class="relative z-10">
                <div class="aspect-square bg-surface-container-low rounded-2xl flex items-center justify-center">
                  <div class="text-center">
                    <span class="icon-[tabler--router] text-8xl text-primary/30 mb-4"></span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.how_work.79') }</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant max-w-xs">
                      { $t('landing.auto.how_work.80') }
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Pricing / License -->
      <section id="pricing" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{ $t('landing.auto.how_work.81') }</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
{ $t('landing.auto.how_work.82') }
            </p>
          </div>
          <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
            <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.how_work.83') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">{ $t('landing.auto.how_work.84') }</p>
              <div class="mb-6">
                <span class="text-4xl font-bold text-on-surface font-display-xl">{ $t('landing.auto.how_work.85') }</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.86') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.87') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.88') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.89') }</span></li>
              </ul>
              <a href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank" class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                { $t('landing.auto.how_work.90') }
              </a>
            </div>
            <div class="bg-primary text-on-primary border border-primary rounded-3xl p-8 flex flex-col relative shadow-xl shadow-primary/20 transform md:-translate-y-4">
              <div class="absolute top-0 right-8 transform -translate-y-1/2 bg-tertiary-container text-on-tertiary-container px-3 py-1 rounded-full font-label-sm text-label-sm">
                { $t('landing.auto.how_work.91') }
              </div>
              <h3 class="font-headline-lg text-headline-lg mb-2">{ $t('landing.auto.how_work.92') }</h3>
              <p class="font-body-md text-body-md text-primary-fixed mb-6">{ $t('landing.auto.how_work.93') }</p>
              <div class="mb-6">
                <span class="text-4xl font-bold font-display-xl">{ $t('landing.auto.how_work.94') }</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.95') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.96') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.97') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.98') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.99') }</span></li>
              </ul>
              <button class="w-full py-3 rounded-xl bg-on-primary text-primary font-headline-md text-headline-md hover:bg-white/90 transition-colors">
                { $t('landing.auto.how_work.100') }
              </button>
            </div>
            <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.how_work.101') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">{ $t('landing.auto.how_work.102') }</p>
              <div class="mb-6">
                <span class="text-4xl font-bold text-on-surface font-display-xl">{ $t('landing.auto.how_work.103') }</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.104') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.105') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.106') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.107') }</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">{ $t('landing.auto.how_work.108') }</span></li>
              </ul>
              <button class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                { $t('landing.auto.how_work.109') }
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section id="faq" class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{ $t('landing.auto.how_work.110') }</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              { $t('landing.auto.how_work.111') }
            </p>
          </div>

          <div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-primary"></span>
                { $t('landing.auto.how_work.112') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.113') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-secondary"></span>
                { $t('landing.auto.how_work.114') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.115') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-tertiary"></span>
                { $t('landing.auto.how_work.116') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.117') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-success"></span>
                { $t('landing.auto.how_work.118') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.119') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-warning"></span>
                { $t('landing.auto.how_work.120') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.121') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-info"></span>
                { $t('landing.auto.how_work.122') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.123') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-purple"></span>
                { $t('landing.auto.how_work.124') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.125') }
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-pink"></span>
                { $t('landing.auto.how_work.126') }
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                { $t('landing.auto.how_work.127') }
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section id="cta" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-16 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">{ $t('landing.auto.how_work.128') }</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                { $t('landing.auto.how_work.129') }
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <a href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  <span class="icon-[tabler--rocket]"></span>
                  { $t('landing.auto.how_work.130') }
                </a>
                <a href="#steps" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  <span class="icon-[tabler--player-play]"></span>
                  { $t('landing.auto.how_work.131') }
                </a>
              </div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'landing'
})
</script>

<style>

</style>
