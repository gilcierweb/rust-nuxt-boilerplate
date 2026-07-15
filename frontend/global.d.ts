// Pinia Plugin Persisted State - Proper type declarations
// See: https://pinia-plugin-persistedstate.esm.dev/guide/cookies.html

import type { StorageLike } from 'pinia-plugin-persistedstate';
import type { CookieOptions } from '#app';

// Cookie options for persisted state cookies
// Excludes Nuxt-specific options that don't apply here
type CookieStorageOptions = Omit<
  CookieOptions,
  'default' | 'watch' | 'readonly' | 'filter'
>;

// FlyonUI Static Methods
import type { IStaticMethods } from 'flyonui/flyonui';

declare global {
  interface Window {
    // FlyonUI static methods for component initialization
    HSStaticMethods: IStaticMethods;
  }
}

// Pinia Plugin Persisted State storages
// This matches the actual export from pinia-plugin-persistedstate
declare module 'pinia-plugin-persistedstate' {
  export type CookiesStorageOptions = Omit<
    CookieOptions,
    'default' | 'watch' | 'readonly' | 'filter'
  >;

  export type StorageLike = {
    getItem: (key: string) => string | null;
    setItem: (key: string, value: string) => void;
    removeItem: (key: string) => void;
  };

  export function cookies(options?: CookiesStorageOptions): StorageLike;
  export function localStorage(): StorageLike;
  export function sessionStorage(): StorageLike;

  export const storages: {
    cookies: typeof cookies;
    localStorage: typeof localStorage;
    sessionStorage: typeof sessionStorage;
  };
}

export {};