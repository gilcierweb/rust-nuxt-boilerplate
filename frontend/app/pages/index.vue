<template>
  <div>
    <main class="pt-20">
      <!-- Section 1: Hero -->
      <section id="hero" class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div
            class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 grid lg:grid-cols-2 gap-16 items-center relative z-10">
          <div class="max-w-xl">
            <div
                class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
              <span class="relative flex h-2 w-2"><span
                  class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span><span
                  class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span></span>
              {{ $t('landing.hero.badgeRelease') }}
            </div>
            <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight">
{{ $t('landing.hero.heading') }}
</h1>
            <p class="font-body-lg text-body-lg text-on-surface-variant mb-10">
{{ $t('landing.hero.description') }}
</p>
            <div class="flex flex-wrap items-center gap-4">
              <a
class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2"
                 href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank">{{ $t('landing.hero.viewOnGitHub') }} <span
                  class="icon-[tabler--arrow-right]"></span></a>
              <a
class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50"
                 href="#features">{{ $t('landing.hero.exploreFeatures') }} <span class="icon-[tabler--compass]"></span></a>
            </div>
          </div>
          <div
              class="relative lg:h-[560px] w-full rounded-[2rem] overflow-hidden shadow-2xl shadow-primary/10 border border-outline-variant/20 bg-surface">
            <div class="absolute inset-0 bg-surface-container-low flex flex-col">
              <div class="h-12 border-b border-outline-variant/20 flex items-center px-4 gap-2 bg-surface">
                <div class="w-3 h-3 rounded-full bg-error/50"></div>
                <div class="w-3 h-3 rounded-full bg-tertiary/50"></div>
                <div class="w-3 h-3 rounded-full bg-primary/50"></div>
              </div>
              <div class="flex-1 relative p-6">
                <div
                    class="bg-surface rounded-xl border border-outline-variant/20 h-full flex flex-col overflow-hidden">
                  <div class="p-4 border-b border-outline-variant/20 flex items-center gap-2">
                    <span class="icon-[tabler--code] text-primary"></span>
                    <span class="font-headline-sm text-headline-sm text-on-surface">{ $t('landing.auto.index.1') }</span>
                  </div>
                  { $t('landing.auto.index.2') }
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 2: Stats/Trust Bar -->
      <section id="stats" class="py-12 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <p class="text-center font-label-sm text-label-sm text-outline mb-8 uppercase tracking-widest">
{ $t('landing.auto.index.3') }
</p>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-8 divide-x divide-outline-variant/20">
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">15+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.index.4') }</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">3</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.index.5') }</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">100%</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.index.6') }</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">{ $t('landing.auto.index.7') }</div>
              <div class="font-body-md text-body-md text-on-surface-variant">{ $t('landing.auto.index.8') }</div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 3: Value Proposition -->
      <section id="value" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center max-w-3xl mx-auto mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-6">
{ $t('landing.auto.index.9') }
</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
{ $t('landing.auto.index.10') }
</p>
          </div>
          <div class="grid md:grid-cols-3 gap-8">
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--bolt] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.index.11') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.12') }
</p>
            </div>
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--shield-lock] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.index.13') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.14') }
</p>
            </div>
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--building-estate] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">{ $t('landing.auto.index.15') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.16') }
</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 4: Tech Stack Showcase -->
      <section id="tech-stack" class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div
                  class="absolute inset-0 bg-gradient-to-br from-primary/5 to-secondary/5 flex items-center justify-center">
                <div class="text-center p-8">
<span class="icon-[tabler--file-code-2] text-8xl text-primary/30 mb-4"></span>
                  <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.17') }</h3>
                  <p class="font-body-md text-body-md text-on-surface-variant max-w-xs mx-auto">
{ $t('landing.auto.index.18') }
</p>
</div>
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl">
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">{ $t('landing.auto.index.19') }</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">{ $t('landing.auto.index.20') }</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
{ $t('landing.auto.index.21') }
</p>
              <ul class="space-y-4 mb-8">
                <li class="flex items-start gap-3">
<span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>{ $t('landing.auto.index.22') }</strong> { $t('landing.auto.index.23') }</span>
                </li>
                <li class="flex items-start gap-3">
<span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>{ $t('landing.auto.index.24') }</strong> { $t('landing.auto.index.25') }</span>
                </li>
                <li class="flex items-start gap-3">
<span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>{ $t('landing.auto.index.26') }</strong> { $t('landing.auto.index.27') }</span>
                </li>
                <li class="flex items-start gap-3">
<span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>{ $t('landing.auto.index.28') }</strong> { $t('landing.auto.index.29') }</span>
                </li>
              </ul>
              <a
class="text-primary font-headline-md text-headline-md font-medium hover:underline flex items-center gap-1"
                 href="#tech-details">{ $t('landing.auto.index.30') } <span class="icon-[tabler--chevron-right] text-sm"></span></a>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 5: Feature Grid (Bento) -->
      <section id="features" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16">
<h2 class="font-display-xl text-display-xl text-on-surface mb-4">
{ $t('landing.auto.index.31') }
</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
{ $t('landing.auto.index.32') }
</p>
</div>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 auto-rows-[280px]">
            <div
                class="md:col-span-2 bg-surface border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="relative z-10 w-2/3">
                <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-4">
                  <span class="icon-[tabler--shield]"></span>
</div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.33') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.34') }
</p>
</div>
              <div
                  class="absolute -right-10 -bottom-10 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors"></div>
            </div>
            <div
                class="bg-surface-container-low border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="w-12 h-12 rounded-full bg-secondary/10 flex items-center justify-center text-secondary mb-4">
                <span class="icon-[tabler--shield-cog]"></span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.35') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.36') }
</p>
</div>
            <div
                class="bg-surface-container-low border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="w-12 h-12 rounded-full bg-tertiary/10 flex items-center justify-center text-tertiary mb-4">
                <span class="icon-[tabler--database]"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.37') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.38') }
</p>
</div>
            <div
                class="md:col-span-2 bg-surface border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="relative z-10 w-2/3">
                <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-4">
                  <span class="icon-[tabler--api]"></span>
</div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.39') }</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.40') }
</p>
</div>
              <div
                  class="absolute -right-10 -bottom-10 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors"></div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 6: Architecture Overview -->
      <section id="architecture" class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16 max-w-3xl mx-auto">
<h2
              class="font-display-xl text-display-xl text-on-surface mb-4">
{ $t('landing.auto.index.41') }
</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
{ $t('landing.auto.index.42') }
</p>
          </div>
          <div class="grid md:grid-cols-4 gap-6">
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mx-auto mb-4 text-primary">
                <span class="icon-[tabler--building] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.43') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.44') }
</p>
</div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mx-auto mb-4 text-secondary">
                <span class="icon-[tabler--server] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.45') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.46') }
</p>
</div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mx-auto mb-4 text-tertiary">
                <span class="icon-[tabler--tool] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.47') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.48') }
</p>
</div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mx-auto mb-4 text-primary">
                <span class="icon-[tabler--cloud] text-3xl"></span>
</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.49') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.50') }
</p>
</div>
          </div>
        </div>
      </section>

      <!-- Section 7: Admin Panel Features -->
      <section id="admin" class="py-24 bg-surface-container-low">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div
                  class="absolute inset-0 bg-gradient-to-br from-primary/5 to-secondary/5 flex items-center justify-center p-8">
                <div class="text-center">
<span class="icon-[tabler--layout-dashboard] text-8xl text-primary/30 mb-4"></span>
                  <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.51') }</h3>
                  <p class="font-body-md text-body-md text-on-surface-variant max-w-xs mx-auto">
{ $t('landing.auto.index.52') }
</p>
</div>
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl">
<span
                class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">{ $t('landing.auto.index.53') }</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">
{ $t('landing.auto.index.54') }
</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
{ $t('landing.auto.index.55') }
</p>
              <div class="space-y-6">
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--users]"></span></div>
                  <div>
<h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.index.56') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.57') }
</p>
</div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--history]"></span></div>
                  <div>
<h4 class="font-headline-md text-headline-md text-on-surface mb-1">{ $t('landing.auto.index.58') }</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.59') }
</p>
</div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 8: Developer Experience -->
      <section id="dx" class="py-24 bg-surface border-t border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div class="absolute inset-0 bg-surface-container-high flex items-center justify-center">
{ $t('landing.auto.index.60') }
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl">
<span
                class="font-label-sm text-label-sm text-tertiary tracking-widest uppercase mb-4 block">{ $t('landing.auto.index.61') }</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">{ $t('landing.auto.index.62') }</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
{ $t('landing.auto.index.63') }
</p>
              <div class="grid grid-cols-2 gap-6">
                <div class="bg-surface-container-low p-5 rounded-2xl">
<span
                    class="icon-[tabler--refresh] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">
{ $t('landing.auto.index.64') }
</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">
{ $t('landing.auto.index.65') }
</p>
</div>
                <div class="bg-surface-container-low p-5 rounded-2xl">
<span
                    class="icon-[tabler--bug] text-tertiary mb-2 block"></span>
                  <h4 class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">{ $t('landing.auto.index.66') }</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">
{ $t('landing.auto.index.67') }
</p>
</div>
                <div class="bg-surface-container-low p-5 rounded-2xl">
<span
                    class="icon-[tabler--terminal] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">
{ $t('landing.auto.index.68') }
</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">
{ $t('landing.auto.index.69') }
</p>
</div>
                <div class="bg-surface-container-low p-5 rounded-2xl">
<span
                    class="icon-[tabler--file-text] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">
{ $t('landing.auto.index.70') }
</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">
{ $t('landing.auto.index.71') }
</p>
</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 9: Security Features -->
      <section id="security" class="py-24 bg-surface-container-lowest border-t border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-6 text-center">
<span
            class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">{ $t('landing.auto.index.72') }</span>
          <h2 class="font-display-xl text-display-xl text-on-surface mb-6 max-w-2xl mx-auto">
{ $t('landing.auto.index.73') }
</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-12 max-w-3xl mx-auto">
{ $t('landing.auto.index.74') }
</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🔐</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.75') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.76') }
</p>
</div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                { $t('landing.auto.index.77') }
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🛡️</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.78') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.79') }
</p>
</div>
            <div class="bg-surface border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🔒</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.80') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.81') }
</p>
</div>
          </div>
        </div>
      </section>

      <!-- Section 10: Deployment Ready -->
      <section id="deployment" class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-6 text-center">
<h2
            class="font-display-xl text-display-xl text-on-surface mb-6 max-w-3xl mx-auto">
{ $t('landing.auto.index.82') }
</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-16 max-w-2xl mx-auto">
{ $t('landing.auto.index.83') }
</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🐳</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.84') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.85') }
</p>
</div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                { $t('landing.auto.index.86') }
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">☸️</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.87') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.88') }
</p>
</div>
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🌐</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.89') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
{ $t('landing.auto.index.90') }
</p>
</div>
          </div>
        </div>
      </section>

      <!-- Section 11: Pricing / License -->
      <section id="license" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16">
<h2 class="font-display-xl text-display-xl text-on-surface mb-4">
{ $t('landing.auto.index.91') }
</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">
{ $t('landing.auto.index.92') }
</p>
            <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
              <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
<h3
                  class="font-headline-lg text-headline-lg text-on-surface mb-2">
{ $t('landing.auto.index.93') }
</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-6">
{ $t('landing.auto.index.94') }
</p>
                <div class="mb-6"><span class="text-4xl font-bold text-on-surface font-display-xl">{ $t('landing.auto.index.95') }</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.96') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.97') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.98') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.99') }</span>
</li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors"
                    disabled>
{ $t('landing.auto.index.100') }
                </button>
              </div>
              <div
                  class="bg-primary text-on-primary border border-primary rounded-3xl p-8 flex flex-col relative shadow-xl shadow-primary/20 transform md:-translate-y-4">
                <div
                    class="absolute top-0 right-8 transform -translate-y-1/2 bg-tertiary-container text-on-tertiary-container px-3 py-1 rounded-full font-label-sm text-label-sm">
                  { $t('landing.auto.index.101') }
                </div>
                <h3 class="font-headline-lg text-headline-lg mb-2">{ $t('landing.auto.index.102') }</h3>
                <p class="font-body-md text-body-md text-primary-fixed mb-6">
{ $t('landing.auto.index.103') }
</p>
                <div class="mb-6"><span class="text-4xl font-bold font-display-xl">{ $t('landing.auto.index.104') }</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.105') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.106') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.107') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.108') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.109') }</span>
</li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl bg-on-primary text-primary font-headline-md text-headline-md hover:bg-white/90 transition-colors">
                  { $t('landing.auto.index.110') }
                </button>
              </div>
              <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
<h3
                  class="font-headline-lg text-headline-lg text-on-surface mb-2">
{ $t('landing.auto.index.111') }
</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-6">
{ $t('landing.auto.index.112') }
</p>
                <div class="mb-6"><span class="text-4xl font-bold text-on-surface font-display-xl">{ $t('landing.auto.index.113') }</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.114') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.115') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.116') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.117') }</span>
</li>
                  <li class="flex items-center gap-3">
<span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">{ $t('landing.auto.index.118') }</span>
</li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                  { $t('landing.auto.index.119') }
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 12: Getting Started -->
      <section id="getting-started" class="py-24 bg-primary text-on-primary">
        <div class="max-w-[1440px] mx-auto px-6 grid lg:grid-cols-2 gap-16 items-center">
          <div class="max-w-xl">
<span
              class="font-label-sm text-label-sm text-primary-fixed-dim tracking-widest uppercase mb-4 block">{ $t('landing.auto.index.120') }</span>
            <h2 class="font-display-xl text-display-xl mb-6 text-on-primary">{ $t('landing.auto.index.121') }</h2>
            <p class="font-body-lg text-body-lg text-primary-fixed mb-8">
{ $t('landing.auto.index.122') }
</p>
            { $t('landing.auto.index.123') }
          </div>
          <div class="relative h-[500px] w-full flex justify-center items-center">
            <div
                class="relative w-64 h-[480px] bg-surface rounded-[3rem] border-8 border-surface-container-highest shadow-2xl overflow-hidden z-10 transform -rotate-6">
              <div class="w-full h-full bg-surface p-4 pt-8">
                <div class="w-20 h-1 bg-outline-variant/30 rounded-full mx-auto mb-6"></div>
                <div class="h-24 bg-primary/10 rounded-xl mb-4"></div>
                <div class="space-y-3">
                  <div class="h-12 bg-surface-container rounded-lg"></div>
                  <div class="h-12 bg-surface-container rounded-lg"></div>
                  <div class="h-12 bg-surface-container rounded-lg"></div>
                </div>
              </div>
              <div
                  class="absolute w-64 h-[480px] bg-surface rounded-[3rem] border-8 border-surface-container-highest shadow-2xl overflow-hidden z-0 transform rotate-6 translate-x-12 translate-y-8 opacity-80">
                <div class="w-full h-full bg-surface p-4 pt-8">
                  <div class="w-20 h-1 bg-outline-variant/30 rounded-full mx-auto mb-6"></div>
                  <div class="h-32 bg-secondary/10 rounded-xl mb-4"></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 13: Community & Ecosystem -->
      <section id="community" class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-6 text-center">
<h2
            class="font-display-xl text-display-xl text-on-surface mb-6 max-w-3xl mx-auto">
{ $t('landing.auto.index.124') }
</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-16 max-w-2xl mx-auto">
{ $t('landing.auto.index.125') }
</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">💬</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.126') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">
{ $t('landing.auto.index.127') }
</p><a class="btn btn-outline btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--message-circle] mr-2"></span>{ $t('landing.auto.index.128') }</a>
</div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                { $t('landing.auto.index.129') }
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🐙</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.130') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">
{ $t('landing.auto.index.131') }
</p><a class="btn btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--code] mr-2"></span>{ $t('landing.auto.index.132') }</a>
</div>
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">📖</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">{ $t('landing.auto.index.133') }</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">
{ $t('landing.auto.index.134') }
</p><a class="btn btn-outline btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--book] mr-2"></span>{ $t('landing.auto.index.135') }</a>
</div>
          </div>
        </div>
      </section>

      <!-- Section 14: Footer CTA -->
      <section id="cta" class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div
              class="bg-surface p-8 md:p-16 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            <div class="relative z-10">
<h2 class="font-display-xl text-display-xl text-on-surface mb-4">
{ $t('landing.auto.index.136') }
</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
{ $t('landing.auto.index.137') }
              </p>
              <div class="flex flex-wrap justify-center gap-4">
<a
                  href="https://github.com/gilcierweb/rust-nuxt-boilerplate" target="_blank"
                  class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2"><span
                  class="icon-[tabler--rocket]"></span>{ $t('landing.auto.index.138') }</a><a
                  href="#getting-started"
                  class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50"><span
                  class="icon-[tabler--player-play]"></span>{ $t('landing.auto.index.139') }</a>
</div>
            </div>
          </div>
        </div>
      </section>
    </main>
  </div>
</template>

<script setup lang="ts">

// Unique component name: NuxtPage collects keepalive page names globally, so
// two pages sharing the inferred filename name break KeepAlive routing.
defineOptions({ name: 'LandingHomePage' })

definePageMeta({
  layout: 'landing'
})
</script>

<style>

</style>
