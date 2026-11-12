import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const ADMIN_FILES = [
  'admin-audit.json',
  'admin-modules.json',
  'admin-permissions.json',
  'admin-roles.json',
  'admin-resources.json',
]

// The namespaces the admin RBAC screens resolve. Every key of a screen must
// exist in every locale, otherwise the interface prints the raw key. The
// dashboard namespace is out of the contract on purpose: it keeps telemetry
// keys only the pt-BR dashboard renders, a drift of its own.
const RBAC_NAMESPACES = [
  'admin.roles',
  'admin.users',
  'admin.permissions',
  'admin.auditLogs',
  'admin.sidebar',
  'admin.common',
  'admin.statusLabels',
]

function flatKeys(node: unknown, prefix = ''): string[] {
  if (node && typeof node === 'object' && !Array.isArray(node)) {
    return Object.entries(node as Record<string, unknown>).flatMap(([key, value]) =>
      flatKeys(value, prefix ? `${prefix}.${key}` : key),
    )
  }
  return [prefix]
}

function keysOf(locale: string): string[] {
  const keys = new Set<string>()
  for (const file of ADMIN_FILES) {
    const document = JSON.parse(
      readFileSync(resolve(__dirname, `../../../../i18n/locales/${locale}/${file}`), 'utf-8'),
    ) as Record<string, unknown>
    for (const key of flatKeys(document)) {
      if (RBAC_NAMESPACES.some((namespace) => key.startsWith(`${namespace}.`))) keys.add(key)
    }
  }
  return [...keys].sort()
}

describe('admin namespaces key parity', () => {
  const ptBR = keysOf('pt-BR')
  const en = keysOf('en')
  const es = keysOf('es')

  it('translates every key the screens use in the three locales', () => {
    expect(es.filter((key) => !en.includes(key))).toEqual([])
    expect(en.filter((key) => !es.includes(key))).toEqual([])
    expect(ptBR.filter((key) => !en.includes(key))).toEqual([])
    expect(en.filter((key) => !ptBR.includes(key))).toEqual([])
  })
})
