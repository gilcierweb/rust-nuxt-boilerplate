<template>
  <div>
    <main class="pt-20">
      <!-- Hero Section -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32">
        <div class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 text-center relative z-10">
          <div class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
            <span class="icon-[tabler--player-play] text-sm"></span>
            Getting Started Guide
          </div>
          <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight max-w-4xl mx-auto">
            From Zero to Production in Minutes
          </h1>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-10 max-w-2xl mx-auto">
            Follow these 5 simple steps to have a fully functional full-stack application with authentication, admin panel, and type-safe API.
          </p>
        </div>
      </section>

      <!-- Steps Section -->
      <section class="py-24 bg-surface" id="steps">
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
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Clone & Install</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Clone the repository and start the full stack with Docker Compose.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Git clone the repo
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Docker Compose up
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Auto-migrations run
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
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Configure Environment</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Set up your environment variables for database, auth, and external services.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    Copy .env.example
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    Set DATABASE_URL
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-secondary text-sm"></span>
                    Configure JWT secrets
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
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Create Admin User</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Run the seed script to create your first admin account with secure credentials.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    Run seed command
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    Admin user created
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-tertiary text-sm"></span>
                    Default roles assigned
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
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Explore Admin Panel</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Log in and explore the dashboard, user management, audit logs, and system health.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    Dashboard with metrics
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    User & role management
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-success text-sm"></span>
                    Audit logs & health
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
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Start Building</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">
                  Your foundation is ready. Add your domain logic, deploy, and ship features.
                </p>
                <ul class="mt-4 space-y-2 font-body-sm text-body-sm text-on-surface-variant">
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Add domain models
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Create API endpoints
                  </li>
                  <li class="flex items-center gap-2">
                    <span class="icon-[tabler--check] text-primary text-sm"></span>
                    Deploy to production
                  </li>
                </ul>              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Deep Dive Features -->
      <section class="py-24 bg-surface" id="features">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12 md:mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Key Capabilities</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Everything you need to build a modern full-stack application, pre-configured and production-ready.
            </p>
          </div>

          <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--shield] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Authentication System</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                JWT access/refresh tokens, Paseto for email verification, PBKDF2 password hashing, TOTP 2FA, OAuth2 providers ready.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--shield-cog] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">RBAC Authorization</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Resource-level permissions, role hierarchies, resource scoping, grant-based authorization with visual editor.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--database] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Type-Safe Database</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Diesel ORM with compile-time SQL verification, migrations, connection pooling, repository pattern, and query builder.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--api] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">OpenAPI + Type Gen</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Auto-generated OpenAPI 3.1 spec with TypeScript client types. Zero-drift API contracts between backend and frontend.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--chart-bar] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Admin Dashboard</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Real-time metrics, user management, role editor, audit logs, system health, and Prometheus metrics endpoint.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--file-code-2] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Developer Experience</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Hot reload for Rust + Vue, VS Code debug configs, auto-generated types, CLI tools for migrations/seeds, Swagger UI.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--shield-lock] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Data Protection</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Field-level encryption, blind indexes for PII, key rotation, audit trails, GDPR/LGPD compliance helpers.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--cloud] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Deployment Ready</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Multi-stage Docker builds, health checks, graceful shutdown, Kubernetes configs, any cloud provider.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30 hover:shadow-lg transition-all duration-300">
              <div class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--puzzle] text-3xl"></span>
              </div>
              <h3 class="font-headline-md text-headline-md text-on-surface mb-2">Extensible Core</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Plugin architecture, custom middleware, event system, background jobs, WebSocket server, modular architecture.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- Architecture Overview -->
      <section class="py-24 bg-surface-container-lowest" id="architecture">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Clean Architecture</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Layers That Scale.</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
                Domain-driven design with clear boundaries. Each layer has a single responsibility and can be tested in isolation.
              </p>
              <div class="space-y-6">
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--building]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">Domain Layer</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Entities, value objects, domain events, business rules. Pure Rust, zero dependencies.</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--server]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">Data Layer</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Diesel repositories, migrations, connection pooling, query builder. Type-safe SQL.</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--tool]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">Application Layer</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Services, use cases, DTOs, transaction management. Orchestrates domain and data.</p>
                  </div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--cloud]"></span></div>
                  <div>
                    <h4 class="font-headline-md text-headline-md text-on-surface mb-1">Interface Layer</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Actix Web controllers, middleware, OpenAPI docs, WebSocket server, Nuxt frontend.</p>
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
                      <circle cx="200" cy="200" r="60" fill="#E3F2FD" stroke="#1976D2" stroke-width="2"/>
                      <text x="200" y="205" text-anchor="middle" font-size="14" fill="#1976D2" font-weight="bold" font-family="system-ui">Domain</text>
                      <text x="200" y="220" text-anchor="middle" font-size="11" fill="#1976D2" font-family="system-ui">Entities • Rules</text>
                      
                      <!-- Data Layer -->
                      <ellipse cx="200" cy="200" rx="110" ry="90" fill="none" stroke="#388E3C" stroke-width="2" stroke-dasharray="5,5"/>
                      <text x="200" y="110" text-anchor="middle" font-size="12" fill="#388E3C" font-weight="bold" font-family="system-ui">Data Layer</text>
                      <text x="200" y="125" text-anchor="middle" font-size="10" fill="#388E3C" font-family="system-ui">Repositories • Migrations</text>
                      
                      <!-- Application Layer -->
                      <ellipse cx="200" cy="200" rx="160" ry="130" fill="none" stroke="#F57C00" stroke-width="2" stroke-dasharray="5,5"/>
                      <text x="200" y="320" text-anchor="middle" font-size="12" fill="#F57C00" font-weight="bold" font-family="system-ui">Application Layer</text>
                      <text x="200" y="335" text-anchor="middle" font-size="10" fill="#F57C00" font-family="system-ui">Services • DTOs • Transactions</text>
                      
                      <!-- Interface Layer -->
                      <ellipse cx="200" cy="200" rx="200" ry="165" fill="none" stroke="#7B1FA2" stroke-width="2" stroke-dasharray="5,5"/>
                      <text x="380" y="205" text-anchor="middle" font-size="12" fill="#7B1FA2" font-weight="bold" font-family="system-ui">Interface</text>
                      <text x="380" y="220" text-anchor="middle" font-size="10" fill="#7B1FA2" font-family="system-ui">API • WebSocket • UI</text>
                      
                      <!-- Arrows showing dependency direction -->
                      <path d="M260 200 L300 200" stroke="#7B1FA2" stroke-width="2" marker-end="url(#arrowhead)"/>
                      <path d="M140 200 L100 200" stroke="#7B1FA2" stroke-width="2" marker-end="url(#arrowhead)"/>
                      <path d="M200 140 L200 100" stroke="#388E3C" stroke-width="2" marker-end="url(#arrowhead)"/>
                      <path d="M200 260 L200 300" stroke="#F57C00" stroke-width="2" marker-end="url(#arrowhead)"/>
                      
                      <defs>
                        <marker id="arrowhead" markerWidth="10" markerHeight="7" refX="9" refY="3.5" orient="auto">
                          <polygon points="0 0, 10 3.5, 0 7" fill="#7B1FA2"/>
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
      <section class="py-24 bg-surface border-y border-outline-variant/20" id="integrations">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div>
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Integrations</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Connect with Your Favorite Tools</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">
                Pre-built integrations and easy extensibility for the services you already use.
              </p>
              
              <div class="space-y-4">
                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="icon-[tabler--credit-card] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">Payment Gateways</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Stripe, MercadoPago, PagSeguro, PayPal ready</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-secondary/10 flex items-center justify-center text-secondary">
                    <span class="icon-[tabler--message] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">SMS & WhatsApp</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Twilio, Vonage, Zenvia, WhatsApp Business API</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-tertiary/10 flex items-center justify-center text-tertiary">
                    <span class="icon-[tabler--building-bank] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">Accounting & ERP</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">Export to ContaAzul, QuickBooks, Xero, NFe</p>
                  </div>
                </div>

                <div class="flex items-center gap-4 p-4 bg-surface rounded-xl border border-outline-variant/30">
                  <div class="w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center text-primary">
                    <span class="icon-[tabler--cloud] text-2xl"></span>
                  </div>
                  <div>
                    <h4 class="font-headline-sm text-headline-sm text-on-surface">Storage Providers</h4>
                    <p class="font-body-sm text-body-sm text-on-surface-variant">AWS S3, Cloudflare R2, Backblaze B2, MinIO</p>
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
                    <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Open API</h3>
                    <p class="font-body-md text-body-md text-on-surface-variant max-w-xs">
                      Build custom integrations with our REST API and auto-generated TypeScript client.
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Pricing / License -->
      <section class="py-24 bg-surface-container-lowest" id="pricing">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Simple, Transparent Licensing</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">Choose the option that fits your needs. All tiers include the complete source code.
            </p>
          </div>
          <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
            <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Community</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">For individuals and open source projects.</p>
              <div class="mb-6">
                <span class="text-4xl font-bold text-on-surface font-display-xl">FREE</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">Full source code access</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">All core features included</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">Community Discord support</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">MIT License - commercial use OK</span></li>
              </ul>
              <a href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank" class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                View on GitHub
              </a>
            </div>
            <div class="bg-primary text-on-primary border border-primary rounded-3xl p-8 flex flex-col relative shadow-xl shadow-primary/20 transform md:-translate-y-4">
              <div class="absolute top-0 right-8 transform -translate-y-1/2 bg-tertiary-container text-on-tertiary-container px-3 py-1 rounded-full font-label-sm text-label-sm">
                Recommended
              </div>
              <h3 class="font-headline-lg text-headline-lg mb-2">Pro Support</h3>
              <p class="font-body-md text-body-md text-primary-fixed mb-6">For teams shipping production applications.</p>
              <div class="mb-6">
                <span class="text-4xl font-bold font-display-xl">Contact Us</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">Priority email support</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">Dedicated Slack channel</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">Security advisory notifications</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">Migration assistance</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span class="font-body-md text-sm">Architecture review (annual)</span></li>
              </ul>
              <button class="w-full py-3 rounded-xl bg-on-primary text-primary font-headline-md text-headline-md hover:bg-white/90 transition-colors">
                Get Pro Support
              </button>
            </div>
            <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col">
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Enterprise</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">For organizations with custom requirements.</p>
              <div class="mb-6">
                <span class="text-4xl font-bold text-on-surface font-display-xl">Custom</span>
              </div>
              <ul class="space-y-4 mb-8 flex-1">
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">SLA with guaranteed response times</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">Custom feature development</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">Security audit assistance</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">On-premise deployment support</span></li>
                <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span class="font-body-md text-sm">Team training & workshops</span></li>
              </ul>
              <button class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                Contact Sales
              </button>
            </div>
          </div>
        </div>
      </section>

      <!-- FAQ Section -->
      <section class="py-24 bg-surface border-y border-outline-variant/20" id="faq">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-12">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Frequently Asked Questions</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">
              Quick answers to common questions about the boilerplate.
            </p>
          </div>

          <div class="grid md:grid-cols-2 gap-6 max-w-4xl mx-auto">
            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-primary"></span>
                Can I use this for commercial projects?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Yes! The boilerplate is MIT licensed. You can use it commercially, modify it, distribute it, and even sell products built with it without any attribution requirements.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-secondary"></span>
                What Rust version is required?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Rust 1.75+ (stable). We track the latest stable Rust release and update the MSRV (Minimum Supported Rust Version) conservatively.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-tertiary"></span>
                Can I use a different database?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Diesel supports PostgreSQL, MySQL, and SQLite. The boilerplate uses PostgreSQL by default but can be adapted. Diesel's query builder abstracts most differences.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-success"></span>
                How does the authentication work?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                JWT access tokens (short-lived) + Paseto refresh tokens (long-lived, revocable). PBKDF2 for password hashing. Optional TOTP 2FA. OAuth2 providers can be added via the auth service.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-warning"></span>
                Can I remove features I don't need?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Absolutely. The modular architecture makes it easy to remove unused modules (e.g., WebSocket server, admin panel, specific API routes) without affecting the core.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-info"></span>
                How do I deploy to production?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Use the provided Docker Compose or Kubernetes manifests. The multi-stage Dockerfile creates minimal images. Health checks, graceful shutdown, and env-based config are built-in.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-purple"></span>
                Is there a migration guide from v0.x?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                Yes, see the MIGRATION.md file in the repository. The v1.0 release includes breaking changes for improved architecture, but we provide step-by-step upgrade instructions.
              </p>
            </div>

            <div class="bg-surface p-6 rounded-2xl border border-outline-variant/30">
              <h3 class="font-headline-md text-headline-md text-on-surface mb-3 flex items-center gap-2">
                <span class="icon-[tabler--help] text-pink"></span>
                How can I contribute?
              </h3>
              <p class="font-body-md text-body-md text-on-surface-variant">
                We welcome contributions! See CONTRIBUTING.md for guidelines. Areas: bug fixes, new features, documentation, translations, security audits, and testing improvements.
              </p>
            </div>
          </div>
        </div>
      </section>

      <!-- CTA Section -->
      <section class="py-24 bg-surface-container-lowest" id="cta">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="bg-surface p-8 md:p-16 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            
            <div class="relative z-10">
              <h2 class="font-display-xl text-display-xl text-on-surface mb-4">Ready to Start Building?</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">
                Clone the repository, follow the quick start guide, and have a running full-stack application in minutes.
              </p>
              <div class="flex flex-wrap justify-center gap-4">
                <a href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank" class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2">
                  <span class="icon-[tabler--rocket]"></span>
                  Star on GitHub
                </a>
                <a href="#steps" class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50">
                  <span class="icon-[tabler--player-play]"></span>
                  View Quick Start
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
