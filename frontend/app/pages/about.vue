<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div
            class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div
              class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="material-symbols-outlined text-sm" data-icon="info">info</span>
            About This Boilerplate
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            A Modern Foundation for Full-Stack Applications
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Built by developers, for developers. This boilerplate eliminates months of setup work so you can focus on
            building your product.
          </p>
        </div>
      </section>

      <!-- Mission, Vision, Values -->
      <section class="py-24 bg-surface">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid md:grid-cols-3 gap-6 lg:gap-8">
            <!-- Mission -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="material-symbols-outlined text-3xl" data-icon="rocket_launch">rocket_launch</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Mission</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Empower developers to ship production-ready full-stack applications in days, not months, by providing a
                battle-tested, type-safe foundation.
              </p>
            </div>

            <!-- Vision -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="material-symbols-outlined text-3xl" data-icon="visibility">visibility</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Vision</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Become the standard starting point for Rust + Nuxt applications, recognized for reliability, developer
                experience, and architectural excellence.
              </p>
            </div>

            <!-- Values -->
            <div
                class="bg-surface-container-lowest p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="material-symbols-outlined text-3xl" data-icon="verified">verified</span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Values</h3>
              <ul class="space-y-2 font-body-md text-body-md text-on-surface-variant">
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Type safety across the stack
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Security by default
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Clean architecture principles
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Developer experience first
                </li>
                <li class="flex items-center gap-2">
                  <span class="material-symbols-outlined text-primary text-sm" data-icon="check">check</span>
                  Open source sustainability
                </li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      <!-- Timeline / Journey -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Project Milestones</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              From initial concept to production-ready boilerplate.
            </p>
          </div>

          <div class="relative">
            <!-- Timeline Line -->
            <div
                class="absolute left-1/2 transform -translate-x-1/2 h-full w-0.5 bg-outline-variant/50 hidden md:block"></div>

            <div class="space-y-12">
              <!-- 2024 Q1 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">2024 Q1</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Project Inception</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Identified the need for a Rust + Nuxt boilerplate after building multiple full-stack apps from
                      scratch.
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2024 Q2 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-secondary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-secondary tracking-widest uppercase">2024 Q2</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Core Architecture</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Designed clean architecture layers: domain, data, application, interface. Implemented Diesel +
                      Actix Web foundation.
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2024 Q3 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-tertiary tracking-widest uppercase">2024 Q3</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Frontend Integration</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Built Nuxt 4 + Vue 3 frontend with TypeScript, Tailwind 4, Pinia, and auto-generated API types.
                    </p>
                  </div>
                </div>
                <div class="w-4 h-4 rounded-full bg-tertiary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:pl-12 hidden md:block"></div>
              </div>

              <!-- 2024 Q4 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:pr-12 hidden md:block"></div>
                <div class="w-4 h-4 rounded-full bg-primary border-4 border-surface z-10"></div>
                <div class="md:w-1/2 md:text-left md:pl-12">
                  <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block">
                    <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase">2024 Q4</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Auth & Admin Panel</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Implemented JWT/Paseto auth, RBAC permissions, audit logging, admin dashboard, and OpenAPI
                      generation.
                    </p>
                  </div>
                </div>
              </div>

              <!-- 2025 Q1 -->
              <div class="relative flex flex-col md:flex-row items-center gap-8">
                <div class="md:w-1/2 md:text-right md:pr-12">
                  <div
                      class="bg-surface p-6 rounded-2xl border border-outline-variant/30 shadow-sm inline-block text-left md:text-right">
                    <span class="font-label-sm text-label-sm text-success tracking-widest uppercase">2025 Q1</span>
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mt-2">Production Hardening</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant mt-2">
                      Added Docker deployment, CI/CD pipelines, comprehensive tests, seed scripts, and documentation.
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

      <!-- Architecture Principles -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Design Principles</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Every architectural decision follows these guiding principles.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="shield">shield</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Security First</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Encryption, authentication, and authorization built into every layer by default.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-secondary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-secondary" data-icon="code">code</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Type Safety</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                End-to-end type safety from database to UI. Compile-time guarantees prevent runtime errors.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-tertiary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-tertiary"
                      data-icon="architecture">architecture</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Clean Architecture</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Separation of concerns with domain-driven design. Testable, maintainable, scalable.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="speed">speed</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Developer Experience</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Hot reload, auto-generated types, comprehensive tooling. Optimized for productivity.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Tech Stack -->
      <section class="py-24 bg-surface border-y border-outline-variant/20">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Technology Choices</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Each technology selected for specific reasons: performance, safety, ecosystem, and long-term support.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Rust + Actix Web</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Memory safety without GC, async runtime,
                compiled speed. Actix for high-performance HTTP.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Diesel ORM</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Compile-time checked SQL, migrations,
                connection pooling. Type-safe database access.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">PostgreSQL</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Robust, ACID-compliant, JSON support,
                excellent ecosystem. Production-proven at scale.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Redis</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Caching, sessions, rate limiting, WebSocket
                pub/sub. High-performance in-memory store.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Nuxt 4 + Vue 3</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">SSR, auto-imports, file-based routing,
                composables. Modern Vue with TypeScript.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-4 text-tertiary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Tailwind CSS 4</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Utility-first, CSS-first config, lightning
                fast, dark mode, design tokens.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-4 text-primary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Pinia</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Lightweight, type-safe state management.
                DevTools support, SSR compatible.</p>
            </div>

            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-4 text-secondary">
                <svg class="w-8 h-8" viewBox="0 0 24 24" fill="currentColor">
                  <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/>
                </svg>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Docker</h3>
              <p class="font-body-sm text-body-sm text-on-surface-variant">Consistent dev/prod environments, multi-stage
                builds, compose for local stack.</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Team / Contributors -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Maintainers & Contributors</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Open source project maintained by a dedicated team and amazing community contributors.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-8">
            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="person">person</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Core Team</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">Lead Maintainers</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Architecture, security, releases, and community.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-secondary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-secondary" data-icon="group">group</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Contributors</h3>
              <p class="font-body-sm text-body-sm text-secondary mt-1">Code & Documentation</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Features, bug fixes, translations, and improvements.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-24 h-24 rounded-full bg-tertiary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-tertiary" data-icon="bug_report">bug_report</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Security Researchers</h3>
              <p class="font-body-sm text-body-sm text-tertiary mt-1">Responsible Disclosure</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Reporting vulnerabilities to keep the ecosystem safe.
              </p>
            </div>

            <div
                class="bg-surface p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div class="w-24 h-24 rounded-full bg-primary-container/20 mx-auto mb-4 flex items-center justify-center">
                <span class="material-symbols-outlined text-4xl text-primary" data-icon="favorite">favorite</span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface">Sponsors</h3>
              <p class="font-body-sm text-body-sm text-primary mt-1">Financial Support</p>
              <p class="font-body-sm text-body-sm text-on-surface-variant mt-3">
                Enabling sustainable development and infrastructure.
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
              <div class="font-display-xl text-display-xl text-primary mb-2">15+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Built-in Modules</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-secondary mb-2">3</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Architecture Layers</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-tertiary mb-2">100%</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Type-Safe Stack</div>
            </div>
            <div class="text-center">
              <div class="font-display-xl text-display-xl text-success mb-2">MIT</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Open Source License</div>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-24 bg-surface-container-lowest">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div
              class="bg-surface p-8 md:p-12 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>

            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Ready to Build?</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                Clone the repository, follow the quick start guide, and have a running full-stack application in
                minutes.
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <NuxtLink to="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank"
                          class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  View on GitHub <span class="material-symbols-outlined" data-icon="arrow_forward">arrow_forward</span>
                </NuxtLink>
                <NuxtLink to="/contact"
                          class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  Get in Touch
                </NuxtLink>
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