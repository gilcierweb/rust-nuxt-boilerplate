// https://nuxt.com/docs/api/configuration/nuxt-config

import tailwindcss from "@tailwindcss/vite";

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  css: ['~/assets/css/main.css'],

  vite: {
    plugins: [
      tailwindcss(),
    ],
    optimizeDeps: {
      include: [
        'flyonui/flyonui', // CJS
      ]
    }
  },

  modules: ['@pinia/nuxt', 'pinia-plugin-persistedstate/nuxt', '@nuxtjs/i18n', '@vite-pwa/nuxt', 'nuxt-security', '@vee-validate/nuxt'],

  app: {
    head: {
      title: "App Rust Nuxt Boilerplate - A production-ready Rust (Actix Web) + Nuxt 4 boilerplate with authentication, authorization, admin panel, and modern developer experience. Clone, configure, and ship.",
      meta: [
        { charset: "utf-8" },
        { name: "viewport", content: "width=device-width, initial-scale=1" },
        { name: "description", content: "A production-ready Rust (Actix Web) + Nuxt 4 boilerplate with authentication, authorization, admin panel, and modern developer experience. Clone, configure, and ship." },
        { name: "robots", content: "noindex, nofollow" }, // keep off search engines
        { name: "theme-color", content: "#FF6F00" },
      ],
    },
    // pageTransition: { name: 'fade', mode: 'out-in' },
  },

  pwa: {
    registerType: 'autoUpdate',
    manifest: {
      id: '/?source=pwa',
      name: 'App Rust Nuxt Boilerplate',
      short_name: 'App Rust Nuxt Boilerplate',
      description: 'A production-ready Rust (Actix Web) + Nuxt 4 boilerplate with authentication, authorization, admin panel, and modern developer experience. Clone, configure, and ship.',
      start_url: '/?source=twa',
      scope: '/',
      display: 'standalone',
      background_color: '#0A0A0F',
      theme_color: '#00E5FF',
      categories: ['saas', 'monitoring', 'industrial'],
      icons: [
        // regular icons
        { src: '/pwa-icon-192.png', sizes: '192x192', type: 'image/png' },
        { src: '/pwa-icon-512.png', sizes: '512x512', type: 'image/png' },
        // maskable icon required for high-quality install UI
        { src: '/pwa-maskable-512.png', sizes: '512x512', type: 'image/png', purpose: 'maskable' },
      ],
      screenshots: [
        { src: '/screenshot-1.webp', sizes: '1280x720', type: 'image/webp' },
        { src: '/screenshot-2.webp', sizes: '1280x720', type: 'image/webp' },
      ],
    },
    workbox: {
      runtimeCaching: [
        // Example: cache game assets
        {
          urlPattern: /\.(?:png|jpg|jpeg|webp|gif|svg|mp3|wav|ogg|mp4|webm|glb|gltf|bin|ttf|woff2)$/,
          handler: 'CacheFirst',
          options: {
            cacheName: 'assets-cache',
            expiration: { maxEntries: 200, maxAgeSeconds: 60 * 60 * 24 * 30 }, // 30 days
          },
        },
      ],
    },
    devOptions: {
      enabled: false,
    },
  },

security: {
    headers: {
      contentSecurityPolicy: {
        "default-src": ["'self'"],
        "img-src": ["'self'", "data:", "blob:"],
        "font-src": ["'self'", "fonts.gstatic.com", "data:"],
        "style-src": ["'self'", "'unsafe-inline'", "fonts.googleapis.com"],
        "script-src": ["'self'", "'unsafe-inline'"],
        "script-src-attr": ["'none'"],
      },
    },
    csrf: true,
  },

// -- Runtime Config
  runtimeConfig: {
    // Server-only (private)
    backendApiBase: process.env.BACKEND_API_BASE || "http://localhost:8080/api/v1",
    backendApiKey: process.env.BACKEND_API_KEY || "",
    // Public (exposed to client) - NEVER put secrets here
    public: {
      // @ts-ignore
      apiBase: "/api/v1", // Always use relative URL for proxy (recommended)
      // @ts-ignore
      wsBase: process.env.NUXT_PUBLIC_WS_BASE || "ws://localhost:8080/api/v1",
      // @ts-ignore
      cdnUrl: process.env.NUXT_PUBLIC_CDN_URL || "https://cdn.rust-nuxt-boilerplate.com",
      // @ts-ignore
      stripeKey: process.env.NUXT_PUBLIC_STRIPE_KEY || "",
      // @ts-ignore
      appName: process.env.NUXT_PUBLIC_APP_NAME || "App Rust Nuxt Boilerplate",
      // @ts-ignore
      // Optional: Direct backend URL for CSR requests (bypasses Nitro proxy).
      // Leave empty to use relative proxy (recommended for production).
      // WARNING: Setting this exposes backend URL in client bundle.
      apiDirectBase: process.env.NUXT_PUBLIC_API_BASE || "",
    },
  },

  i18n: {
    langDir: "locales",
    locales: [
      {
        code: "en",
        iso: "en-US",
        name: "English",
        files: [
          "en.json",
          "en/common.json",
          "en/auth.json",
          "en/landing.json",
          "en/portal.json",
          "en/seed.json",
          "en/admin-audit.json",
          "en/admin-roles.json",
          "en/admin-resources.json",
        ],
      },
      {
        code: "es",
        iso: "es-ES",
        name: "Español",
        files: [
          "es.json",
          "es/common.json",
          "es/auth.json",
          "es/landing.json",
          "es/seed.json",
          "es/admin-roles.json",
          "es/admin-resources.json",
        ],
      },
      {
        code: "pt-BR",
        iso: "pt-BR",
        name: "Português Brasil",
        files: [
          "pt-BR.json",
          "pt-BR/common.json",
          "pt-BR/auth.json",
          "pt-BR/landing.json",
          "pt-BR/portal.json",
          "pt-BR/seed.json",
          "pt-BR/admin-audit.json",
          "pt-BR/admin-roles.json",
          "pt-BR/admin-resources.json",
          "pt-BR/admin-modules.json",
          "pt-BR/portal-modules.json",
        ],
      },
      {
        code: "pt",
        iso: "pt-BR",
        files: [
          "pt-BR.json",
          "pt-BR/common.json",
          "pt-BR/auth.json",
          "pt-BR/landing.json",
          "pt-BR/portal.json",
          "pt-BR/seed.json",
          "pt-BR/admin-audit.json",
          "pt-BR/admin-roles.json",
          "pt-BR/admin-resources.json",
          "pt-BR/admin-modules.json",
          "pt-BR/portal-modules.json",
        ],
      },
    ],
    defaultLocale: "pt-BR",
    strategy: "prefix_except_default",
    lazy: true,
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: "i18n_redirected",
      redirectOn: "root",
    },
  },

  nitro: {
    compressPublicAssets: true,
    routeRules: {
      '/api/v1/**': {
        security: {
          rateLimiter: false,
        },
        csurf: false,
      },
    },
  },

  ssr: true,
});
