// The injections this project provides.
//
// The Nuxt type generation emits no declaration for the `$api` injection the
// api plugin provides, so every `useNuxtApp().$api` read as `unknown`. The
// shape is declared once here: a typed fetch that carries the auth headers, the
// cookies and the refresh retry the plugin builds.

declare module '#app/nuxt' {
  interface NuxtApp {
    /** The typed API call the plugin provides, with the auth plumbing. */
    $api: <T = unknown>(
      request: string,
      options?: Record<string, any>,
    ) => Promise<T>
  }
}

export {}
