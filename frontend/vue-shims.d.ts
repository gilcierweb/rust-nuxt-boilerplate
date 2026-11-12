// The shims the type checker needs for the module kinds this project imports.
//
// A `.vue` file is a component, and the checker cannot read the file itself:
// the shim types every single file component import as a component, which is
// what the tooling already does at build time.

declare module '*.vue' {
  import type { DefineComponent } from 'vue'

  const component: DefineComponent<Record<string, unknown>, Record<string, unknown>, unknown>
  export default component
}
