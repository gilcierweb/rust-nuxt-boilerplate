import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { extname, join, resolve } from 'node:path'

// The translation keys the live code references, verified against every
// locale the app loads. The parity test next door compares the keys the
// locales define; this one closes the other side of the contract: a key the
// code calls that no locale defines prints the raw key to the user, in every
// language. `auth.register.error.emailExists` was exactly that bug.

const ROOT = process.cwd()

const SOURCE_DIRECTORIES = ['app', 'shared', 'server']

const SOURCE_EXTENSIONS = new Set(['.ts', '.vue'])

const LOCALES = ['en', 'es', 'pt-BR']

function sourceFiles(directory: string): string[] {
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return sourceFiles(path)
    if (!SOURCE_EXTENSIONS.has(extname(entry.name))) return []
    return [path]
  })
}

// The two call shapes the codebase uses: the `$t` of a template and the `t`
// destructured from useI18n in a script. Keys assembled at runtime are not
// statically verifiable and stay out of the contract.
const TRANSLATION_CALL = /(?:\$t|\bt)\(\s*['"]([\w.-]+)['"]/g

function referencedKeys(): string[] {
  const keys = new Set<string>()
  for (const directory of SOURCE_DIRECTORIES) {
    for (const file of sourceFiles(join(ROOT, directory))) {
      const content = readFileSync(file, 'utf-8')
      for (const match of content.matchAll(TRANSLATION_CALL)) {
        keys.add(match[1]!)
      }
    }
  }
  return [...keys].sort()
}

function flatKeys(node: unknown, prefix = ''): string[] {
  if (node && typeof node === 'object' && !Array.isArray(node)) {
    return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
      flatKeys(value, prefix ? `${prefix}.${key}` : key),
    )
  }
  return [prefix]
}

function definedKeys(locale: string): Set<string> {
  const keys = new Set<string>()
  const localeDirectory = join(ROOT, 'i18n/locales', locale)
  if (!existsSync(localeDirectory)) return keys
  for (const entry of readdirSync(localeDirectory)) {
    if (!entry.endsWith('.json')) continue
    if (entry === `${locale}.json`) continue
    const document = JSON.parse(readFileSync(join(localeDirectory, entry), 'utf-8'))
    for (const key of flatKeys(document)) keys.add(key)
  }
  return keys
}

describe('translation keys the code references', () => {
  const referenced = referencedKeys()
  const defined = Object.fromEntries(
    LOCALES.map((locale) => [locale, definedKeys(locale)]),
  ) as Record<string, Set<string>>

  it('finds the keys to verify', () => {
    expect(referenced.length).toBeGreaterThan(100)
  })

  it.each(LOCALES)('defines every referenced key in %s', (locale) => {
    const missing = referenced.filter((key) => !defined[locale]!.has(key))
    expect(missing).toEqual([])
  })
})