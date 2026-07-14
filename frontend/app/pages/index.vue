<template>
  <div>
    <main class="pt-20">
      <!-- Section 1: Hero -->
      <section class="relative bg-surface-container-lowest overflow-hidden py-24 lg:py-32" id="hero">
        <div
            class="absolute inset-0 bg-gradient-to-br from-primary-container/10 to-transparent pointer-events-none"></div>
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8 grid lg:grid-cols-2 gap-16 items-center relative z-10">
          <div class="max-w-xl">
            <div
                class="inline-flex items-center gap-2 px-3 py-1.5 rounded-full bg-primary-container/10 text-primary-fixed-variant text-label-sm font-label-sm mb-6 border border-primary-container/20">
              <span class="relative flex h-2 w-2"><span
                  class="animate-ping absolute inline-flex h-full w-full rounded-full bg-primary opacity-75"></span><span
                  class="relative inline-flex rounded-full h-2 w-2 bg-primary"></span></span>
              v1.0 Released: Production-Ready Boilerplate
            </div>
            <h1 class="font-display-xl text-display-xl text-on-surface mb-6 leading-tight">Build Full-Stack Apps
              Faster.</h1>
            <p class="font-body-lg text-body-lg text-on-surface-variant mb-10">A production-ready Rust (Actix Web) +
              Nuxt 4 boilerplate with authentication, authorization, admin panel, and modern developer experience.
              Clone, configure, and ship.</p>
            <div class="flex flex-wrap items-center gap-4">
              <a class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2"
                 href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank">View on GitHub <span
                  class="icon-[tabler--arrow-right]"></span></a>
              <a class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50"
                 href="#features">Explore Features <span class="icon-[tabler--compass]"></span></a>
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
                    <span class="font-headline-sm text-headline-sm text-on-surface">main.rs</span>
                  </div>
                  <pre
                      class="flex-1 p-4 overflow-auto font-mono text-sm text-on-surface-variant bg-surface-container-lowest"><code>{`use actix_web::{App, HttpServer, web};
use config::AppConfig;
use db::database::{Database, DBPool};
use repositories::AppContainer;
use routes::router::config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = AppConfig::from_env()?;
    let db = Database::new();
    let pool = db.pool.clone();

    let container = web::Data::new(
        AppContainer::new(pool.clone(), config.clone())
    );

    HttpServer::new(move || {
        App::new()
            .app_data(container.clone())
            .configure(config)
    })
    .bind((config.host, config.port))?
    .run()
    .await
}`}</code></pre>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 2: Stats/Trust Bar -->
      <section class="py-12 bg-surface border-y border-outline-variant/20" id="stats">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <p class="text-center font-label-sm text-label-sm text-outline mb-8 uppercase tracking-widest">Trusted by
            developers worldwide</p>
          <div class="grid grid-cols-2 md:grid-cols-4 gap-8 divide-x divide-outline-variant/20">
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">15+</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Built-in Modules</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">3</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Architecture Layers</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">100%</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Type-Safe</div>
            </div>
            <div class="text-center px-4">
              <div class="font-display-xl text-display-xl text-primary mb-2">MIT</div>
              <div class="font-body-md text-body-md text-on-surface-variant">Open Source</div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 3: Value Proposition -->
      <section class="py-24 bg-surface-container-lowest" id="value">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center max-w-3xl mx-auto mb-16">
            <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Stop Building Boilerplate. Start Building
              Features.</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">Everything you need for a modern full-stack
              app: authentication, RBAC, admin panel, API docs, and type-safe database layer. Pre-configured and
              production-hardened.</p>
          </div>
          <div class="grid md:grid-cols-3 gap-8">
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mb-6 text-primary">
                <span class="icon-[tabler--bolt] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Zero-Config Start</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Clone, run `docker compose up`, and you have
                a working full-stack app with hot-reload in seconds.</p>
            </div>
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mb-6 text-secondary">
                <span class="icon-[tabler--shield-lock] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Secure by Default</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">JWT auth, PBKDF2 password hashing, RBAC
                permissions, CSRF protection, and encrypted sensitive fields out of the box.</p>
            </div>
            <div
                class="bg-surface p-8 rounded-2xl border border-outline-variant/30 hover:shadow-lg hover:-translate-y-1 transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mb-6 text-tertiary">
                <span class="icon-[tabler--building-estate] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-3">Clean Architecture</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Domain-driven design with repositories,
                services, controllers, and middleware separated for maintainability.</p>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 4: Tech Stack Showcase -->
      <section class="py-24 bg-surface-container-low border-y border-outline-variant/20" id="tech-stack">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div
                  class="absolute inset-0 bg-gradient-to-br from-primary/5 to-secondary/5 flex items-center justify-center">
                <div class="text-center p-8"><span class="icon-[tabler--file-code-2] text-8xl text-primary/30 mb-4"></span>
                  <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Full-Stack Type Safety</h3>
                  <p class="font-body-md text-body-md text-on-surface-variant max-w-xs mx-auto">Shared types between
                    Rust backend and TypeScript frontend via OpenAPI generation.</p></div>
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl">
              <span class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Technology Stack</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Modern, Performant, Maintainable.</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">Each technology chosen for developer
                experience, performance, and long-term stability.</p>
              <ul class="space-y-4 mb-8">
                <li class="flex items-start gap-3"><span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>Backend:</strong> Rust + Actix Web + Diesel ORM + PostgreSQL</span>
                </li>
                <li class="flex items-start gap-3"><span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>Frontend:</strong> Nuxt 4 + Vue 3 + TypeScript + Tailwind CSS 4</span>
                </li>
                <li class="flex items-start gap-3"><span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>Auth:</strong> JWT + PBKDF2 + Paseto tokens + RBAC</span>
                </li>
                <li class="flex items-start gap-3"><span class="icon-[tabler--circle-check] text-primary mt-0.5"></span><span
                    class="font-body-md text-body-md text-on-surface"><strong>DevOps:</strong> Docker + Docker Compose + Hot Reload</span>
                </li>
              </ul>
              <a class="text-primary font-headline-md text-headline-md font-medium hover:underline flex items-center gap-1"
                 href="#tech-details">View Full Stack <span class="icon-[tabler--chevron-right] text-sm"></span></a>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 5: Feature Grid (Bento) -->
      <section class="py-24 bg-surface-container-lowest" id="features">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16"><h2 class="font-display-xl text-display-xl text-on-surface mb-4">Everything You
            Need. Nothing You Don't.</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant max-w-2xl mx-auto">A comprehensive toolkit
              presented in a clean, digestible format.</p></div>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6 auto-rows-[280px]">
            <div
                class="md:col-span-2 bg-surface border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="relative z-10 w-2/3">
                <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-4">
                  <span class="icon-[tabler--shield]"></span></div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Complete Auth System</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">JWT access/refresh tokens, Paseto for email
                  verification, PBKDF2 password hashing, 2FA/TOTP, OAuth2 providers ready.</p></div>
              <div
                  class="absolute -right-10 -bottom-10 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors"></div>
            </div>
            <div
                class="bg-surface-container-low border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="w-12 h-12 rounded-full bg-secondary/10 flex items-center justify-center text-secondary mb-4">
                <span class="icon-[tabler--shield-cog]"></span>
              </div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Admin Panel</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">User management, role editor, audit logs,
                metrics dashboard, and system health monitoring.</p></div>
            <div
                class="bg-surface-container-low border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="w-12 h-12 rounded-full bg-tertiary/10 flex items-center justify-center text-tertiary mb-4">
                <span class="icon-[tabler--database]"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Type-Safe Database</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Diesel ORM with compile-time SQL
                verification, migrations, connection pooling, and repository pattern.</p></div>
            <div
                class="md:col-span-2 bg-surface border border-outline-variant/30 rounded-3xl p-8 relative overflow-hidden group hover:shadow-md transition-shadow">
              <div class="relative z-10 w-2/3">
                <div class="w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center text-primary mb-4">
                  <span class="icon-[tabler--api]"></span></div>
                <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">OpenAPI + Type Generation</h3>
                <p class="font-body-md text-body-md text-on-surface-variant">Auto-generated OpenAPI 3.1 spec with
                  TypeScript client types. Zero-drift API contracts.</p></div>
              <div
                  class="absolute -right-10 -bottom-10 w-64 h-64 bg-primary/5 rounded-full blur-3xl group-hover:bg-primary/10 transition-colors"></div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 6: Architecture Overview -->
      <section class="py-24 bg-surface border-y border-outline-variant/20" id="architecture">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16 max-w-3xl mx-auto"><h2
              class="font-display-xl text-display-xl text-on-surface mb-4">Clean Architecture in Action.</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">Domain-driven design with clear boundaries.
              Repositories, services, controllers, and middleware separated for testability and maintainability.</p>
          </div>
          <div class="grid md:grid-cols-4 gap-6">
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mx-auto mb-4 text-primary">
                <span class="icon-[tabler--building] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Domain Layer</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Entities, value objects, domain events, and
                business rules. Pure Rust, zero dependencies.</p></div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-secondary-container/10 flex items-center justify-center mx-auto mb-4 text-secondary">
                <span class="icon-[tabler--server] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Data Layer</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Diesel repositories, migrations, connection
                pooling, and query builder. Type-safe SQL.</p></div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-tertiary-container/10 flex items-center justify-center mx-auto mb-4 text-tertiary">
                <span class="icon-[tabler--tool] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Application Layer</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Services, use cases, DTOs, and transaction
                management. Orchestrates domain and data.</p></div>
            <div
                class="bg-surface-container-lowest p-6 rounded-2xl border border-outline-variant/30 text-center hover:shadow-lg transition-all duration-300">
              <div
                  class="w-14 h-14 rounded-xl bg-primary-container/10 flex items-center justify-center mx-auto mb-4 text-primary">
                <span class="icon-[tabler--cloud] text-3xl"></span></div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Interface Layer</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Actix Web controllers, middleware, OpenAPI
                docs, WebSocket server, and Nuxt frontend.</p></div>
          </div>
        </div>
      </section>

      <!-- Section 7: Admin Panel Features -->
      <section class="py-24 bg-surface-container-low" id="admin">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div
                  class="absolute inset-0 bg-gradient-to-br from-primary/5 to-secondary/5 flex items-center justify-center p-8">
                <div class="text-center"><span class="icon-[tabler--layout-dashboard] text-8xl text-primary/30 mb-4"></span>
                  <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Admin Dashboard</h3>
                  <p class="font-body-md text-body-md text-on-surface-variant max-w-xs mx-auto">Real-time metrics, user
                    management, audit trails, and system health in one place.</p></div>
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl"><span
                class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Admin Capabilities</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Manage Your Application with
                Confidence.</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">Built-in admin panel with role-based
                access, comprehensive audit logging, and real-time metrics.</p>
              <div class="space-y-6">
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--users]"></span></div>
                  <div><h4 class="font-headline-md text-headline-md text-on-surface mb-1">User & Role Management</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Create users, assign roles, configure
                      permissions with visual editor.</p></div>
                </div>
                <div class="bg-surface p-6 rounded-2xl border border-outline-variant/20 shadow-sm flex gap-4">
                  <div class="mt-1 text-secondary"><span class="icon-[tabler--history]"></span></div>
                  <div><h4 class="font-headline-md text-headline-md text-on-surface mb-1">Audit Logging</h4>
                    <p class="font-body-md text-body-md text-on-surface-variant">Full trail of critical actions with
                      actor, resource, changes, and metadata.</p></div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 8: Developer Experience -->
      <section class="py-24 bg-surface border-t border-outline-variant/20" id="dx">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="grid lg:grid-cols-2 gap-16 items-center">
            <div
                class="order-2 lg:order-1 relative rounded-[2rem] overflow-hidden shadow-xl border border-outline-variant/30 aspect-[4/3] bg-surface-container">
              <div class="absolute inset-0 bg-surface-container-high flex items-center justify-center"><pre
                  class="p-6 bg-surface-container-lowest rounded-xl border border-outline-variant/20 w-full max-w-md text-sm overflow-auto text-on-surface-variant"><code>{`# Hot reload in action
cargo watch -x run
# Or with docker
docker compose up --watch`}</code></pre>
              </div>
            </div>
            <div class="order-1 lg:order-2 max-w-xl"><span
                class="font-label-sm text-label-sm text-tertiary tracking-widest uppercase mb-4 block">Developer Experience</span>
              <h2 class="font-display-xl text-display-xl text-on-surface mb-6">Code. Save. See Changes Instantly.</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8">Full hot-reload for both Rust backend
                and Vue frontend. TypeScript types regenerate automatically on API changes.</p>
              <div class="grid grid-cols-2 gap-6">
                <div class="bg-surface-container-low p-5 rounded-2xl"><span
                    class="icon-[tabler--refresh] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">Hot Reload</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">Rust + Vue simultaneous
                    reload</p></div>
                <div class="bg-surface-container-low p-5 rounded-2xl"><span
                    class="icon-[tabler--bug] text-tertiary mb-2 block"></span>
                  <h4 class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">Debug Support</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">VS Code launch configs
                    included</p></div>
                <div class="bg-surface-container-low p-5 rounded-2xl"><span
                    class="icon-[tabler--terminal] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">CLI Tools</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">Migration, seed, codegen
                    commands</p></div>
                <div class="bg-surface-container-low p-5 rounded-2xl"><span
                    class="icon-[tabler--file-text] text-tertiary mb-2 block"></span><h4
                    class="font-headline-md text-headline-md text-on-surface mb-1 text-sm">API Docs</h4>
                  <p class="font-body-md text-body-md text-on-surface-variant text-xs">Swagger UI + Scalar at
                    /swagger-ui</p></div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 9: Security Features -->
      <section class="py-24 bg-surface-container-lowest border-t border-outline-variant/20" id="security">
        <div class="max-w-[1440px] mx-auto px-6 text-center"><span
            class="font-label-sm text-label-sm text-primary tracking-widest uppercase mb-4 block">Security First</span>
          <h2 class="font-display-xl text-display-xl text-on-surface mb-6 max-w-2xl mx-auto">Enterprise-Grade Security
            Built In.</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-12 max-w-3xl mx-auto">No security configuration
            required. Sensible defaults protect your application from day one.</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🔐</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Auth & Tokens</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">JWT access tokens, Paseto refresh tokens,
                PBKDF2 hashing, TOTP 2FA, OAuth2 ready.</p></div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                Recommended
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🛡️</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">RBAC & Permissions</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Resource-level permissions, role hierarchies,
                resource scoping, grant-based authorization.</p></div>
            <div class="bg-surface border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🔒</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Data Protection</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Field-level encryption, blind indexes for
                PII, key rotation, audit trails, GDPR/LGPD ready.</p></div>
          </div>
        </div>
      </section>

      <!-- Section 10: Deployment Ready -->
      <section class="py-24 bg-surface border-y border-outline-variant/20" id="deployment">
        <div class="max-w-[1440px] mx-auto px-6 text-center"><h2
            class="font-display-xl text-display-xl text-on-surface mb-6 max-w-3xl mx-auto">Deploy Anywhere. Scale
          Effortlessly.</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-16 max-w-2xl mx-auto">Docker-first architecture
            with multi-stage builds, health checks, and zero-downtime deployment support.</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🐳</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Docker Native</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Multi-stage builds for minimal images. Docker
                Compose for local dev and production.</p></div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                Production Ready
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">☸️</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Kubernetes Ready</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Health endpoints, graceful shutdown, config
                via env, horizontal scaling support.</p></div>
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🌐</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Any Cloud</h3>
              <p class="font-body-md text-body-md text-on-surface-variant">Deploy to AWS, GCP, Azure, DigitalOcean,
                Fly.io, Railway, or your own servers.</p></div>
          </div>
        </div>
      </section>

      <!-- Section 11: Pricing / License -->
      <section class="py-24 bg-surface-container-lowest" id="license">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div class="text-center mb-16"><h2 class="font-display-xl text-display-xl text-on-surface mb-4">Free. Open
            Source. Forever.</h2>
            <p class="font-body-lg text-body-lg text-on-surface-variant">MIT licensed. No vendor lock-in. No feature
              gating. Use it commercially, modify it, distribute it.</p>
            <div class="grid md:grid-cols-3 gap-8 max-w-5xl mx-auto">
              <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col"><h3
                  class="font-headline-lg text-headline-lg text-on-surface mb-2">MIT License</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-6">Permissive license for any use
                  case.</p>
                <div class="mb-6"><span class="text-4xl font-bold text-on-surface font-display-xl">FREE</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Commercial use allowed</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Modification & distribution</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Private use</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">No attribution required</span></li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors"
                    disabled>View License
                </button>
              </div>
              <div
                  class="bg-primary text-on-primary border border-primary rounded-3xl p-8 flex flex-col relative shadow-xl shadow-primary/20 transform md:-translate-y-4">
                <div
                    class="absolute top-0 right-8 transform -translate-y-1/2 bg-tertiary-container text-on-tertiary-container px-3 py-1 rounded-full font-label-sm text-label-sm">
                  Best Value
                </div>
                <h3 class="font-headline-lg text-headline-lg mb-2">Pro Support</h3>
                <p class="font-body-md text-body-md text-primary-fixed mb-6">For teams shipping production
                  applications.</p>
                <div class="mb-6"><span class="text-4xl font-bold font-display-xl">Contact Us</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">Priority email support</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">Dedicated Slack channel</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">Security advisory notifications</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">Migration assistance</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary-fixed text-sm"></span><span
                      class="font-body-md text-sm">Architecture review (annual)</span></li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl bg-on-primary text-primary font-headline-md text-headline-md hover:bg-white/90 transition-colors">
                  Get Pro Support
                </button>
              </div>
              <div class="bg-surface border border-outline-variant/30 rounded-3xl p-8 flex flex-col"><h3
                  class="font-headline-lg text-headline-lg text-on-surface mb-2">Enterprise</h3>
                <p class="font-body-md text-body-md text-on-surface-variant mb-6">For organizations with custom
                  requirements.</p>
                <div class="mb-6"><span class="text-4xl font-bold text-on-surface font-display-xl">Custom</span></div>
                <ul class="space-y-4 mb-8 flex-1">
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">SLA with guaranteed response times</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Custom feature development</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Security audit assistance</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">On-premise deployment support</span></li>
                  <li class="flex items-center gap-3"><span class="icon-[tabler--check] text-primary text-sm"></span><span
                      class="font-body-md text-sm">Team training & workshops</span></li>
                </ul>
                <button
                    class="w-full py-3 rounded-xl border border-primary text-primary font-headline-md text-headline-md hover:bg-primary/5 transition-colors">
                  Contact Sales
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- Section 12: Getting Started -->
      <section class="py-24 bg-primary text-on-primary" id="getting-started">
        <div class="max-w-[1440px] mx-auto px-6 grid lg:grid-cols-2 gap-16 items-center">
          <div class="max-w-xl"><span
              class="font-label-sm text-label-sm text-primary-fixed-dim tracking-widest uppercase mb-4 block">Quick Start</span>
            <h2 class="font-display-xl text-display-xl mb-6 text-on-primary">Up and Running in 3 Commands.</h2>
            <p class="font-body-lg text-body-lg text-primary-fixed mb-8">Clone the repo, start the stack, and open your
              browser. That's it.</p>
            <pre
                class="bg-surface-container-lowest text-on-surface rounded-xl p-6 overflow-x-auto text-sm font-mono"><code>{`# 1. Clone
git clone https://github.com/your-org/rust-nuxt-boilerplate.git
cd rust-nuxt-boilerplate

# 2. Start (includes DB, Redis, backend, frontend)
docker compose up --build

# 3. Open http://localhost:3000`}</code></pre>
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
      <section class="py-24 bg-surface border-y border-outline-variant/20" id="community">
        <div class="max-w-[1440px] mx-auto px-6 text-center"><h2
            class="font-display-xl text-display-xl text-on-surface mb-6 max-w-3xl mx-auto">Join a Growing Community of
          Builders.</h2>
          <p class="font-body-lg text-body-lg text-on-surface-variant mb-16 max-w-2xl mx-auto">Thousands of developers
            use this boilerplate to ship faster. Contribute, ask questions, share your projects.</p>
          <div class="grid md:grid-cols-3 gap-8">
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">💬</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Discord Community</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">Get help, share ideas, and connect with
                other builders.</p><a class="btn btn-outline btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--message-circle] mr-2"></span>Join Discord</a></div>
            <div
                class="bg-surface border-2 border-primary/20 p-8 rounded-3xl shadow-md relative transform md:-translate-y-4">
              <div
                  class="absolute -top-3 left-1/2 -translate-x-1/2 bg-primary text-on-primary px-3 py-1 rounded-full font-label-sm text-label-sm">
                Most Active
              </div>
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">🐙</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">GitHub Discussions</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">Ask questions, request features, report
                bugs, and contribute code.</p><a class="btn btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--code] mr-2"></span>View Repository</a></div>
            <div class="bg-surface-container-lowest border border-outline-variant/30 p-8 rounded-3xl shadow-sm">
              <div class="text-4xl font-black text-primary mb-4 font-display-xl">📖</div>
              <h3 class="font-headline-lg text-headline-lg text-on-surface mb-2">Documentation</h3>
              <p class="font-body-md text-body-md text-on-surface-variant mb-6">Comprehensive guides, API reference, and
                architecture deep-dives.</p><a class="btn btn-outline btn-primary" href="#" target="_blank"><span
                class="icon-[tabler--book] mr-2"></span>Read Docs</a></div>
          </div>
        </div>
      </section>

      <!-- Section 14: Footer CTA -->
      <section class="py-24 bg-surface-container-lowest" id="cta">
        <div class="max-w-[1440px] mx-auto px-4 sm:px-6 lg:px-8">
          <div
              class="bg-surface p-8 md:p-16 rounded-3xl border border-outline-variant/30 text-center relative overflow-hidden">
            <div class="absolute -right-20 -top-20 w-64 h-64 bg-primary/5 rounded-full blur-3xl"></div>
            <div class="absolute -left-20 -bottom-20 w-64 h-64 bg-secondary/5 rounded-full blur-3xl"></div>
            <div class="relative z-10"><h2 class="font-display-xl text-display-xl text-on-surface mb-4">Ready to Ship
              Faster?</h2>
              <p class="font-body-lg text-body-lg text-on-surface-variant mb-8 max-w-2xl mx-auto">Clone the boilerplate,
                customize for your needs, and deploy your first feature today.
              </p>
              <div class="flex flex-wrap justify-center gap-4"><a
                  href="https://github.com/your-org/rust-nuxt-boilerplate" target="_blank"
                  class="bg-primary text-on-primary px-8 py-4 rounded-xl font-headline-md text-headline-md font-medium hover:bg-primary-container hover:text-on-primary-container hover:-translate-y-0.5 active:scale-95 transition-all shadow-sm shadow-primary/20 flex items-center gap-2"><span
                  class="icon-[tabler--rocket]"></span>Star on GitHub</a><a
                  href="#getting-started"
                  class="px-8 py-4 rounded-xl font-headline-md text-headline-md text-on-surface hover:bg-surface-container transition-colors flex items-center gap-2 border border-outline-variant/50"><span
                  class="icon-[tabler--player-play]"></span>View Quick Start</a></div>
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
