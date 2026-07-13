import type { IStaticMethods } from "flyonui/flyonui";

// Pinia Plugin Persisted State
declare const piniaPluginPersistedstate: {
  cookies: (options?: {
    sameSite?: 'strict' | 'lax' | 'none';
    secure?: boolean;
    maxAge?: number;
    domain?: string;
    expires?: Date;
    httpOnly?: boolean;
    partitioned?: boolean;
    path?: string;
  }) => any;
  localStorage: () => any;
  sessionStorage: () => any;
};

declare global {
  interface Window {
    // Optional third-party libraries
 
    // FlyonUI
    HSStaticMethods: IStaticMethods;
  }
}

export {};