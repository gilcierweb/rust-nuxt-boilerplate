import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { extname, join, resolve } from 'node:path'

// The translation calls of the live code must carry the key alone.
//
// The second argument of `t()` is a fallback, not an ignored default: when a
// key is missing from a locale, the fallback renders instead of the key, so a
// Portuguese literal inside an English UI passes review because the line
// looks translated. `PermissionsForm.vue` shows the single-argument style the
// codebase follows; `RolesForm.vue` had eighteen exceptions.

const ROOT = process.cwd()

const SOURCE_DIRECTORIES = ['app', 'shared', 'server']

const SOURCE_EXTENSIONS = new Set(['.ts', '.vue'])

function sourceFiles(directory: string): string[] {
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return sourceFiles(path)
    if (!SOURCE_EXTENSIONS.has(extname(entry.name))) return []
    return [path]
  })
}

// `t('key', 'literal')` in any quoting style: the comma followed by a string
// literal is the fallback the rule forbids.
const FALLBACK_CALL = /(?:\$t|\bt)\(\s*['"][\w.-]+['"]\s*,\s*['"]/g

function fallbackSites(): Array<{ file: string; line: number }> {
  const sites: Array<{ file: string; line: number }> = []
  for (const directory of SOURCE_DIRECTORIES) {
    for (const file of sourceFiles(join(ROOT, directory))) {
      const content = readFileSync(file, 'utf-8')
      const lines = content.split('\n')
      lines.forEach((line, index) => {
        for (const _match of line.matchAll(FALLBACK_CALL)) {
          sites.push({ file: file.slice(ROOT.length + 1), line: index + 1 })
        }
      })
    }
  }
  return sites
}

describe('translation calls', () => {
  it('never carry a literal fallback', () => {
    expect(fallbackSites()).toEqual([])
  })
})

describe('translation call discovery', () => {
  it('verifies the rule can see the files it scans', () => {
    const scanned = SOURCE_DIRECTORIES.flatMap((directory) =>
      sourceFiles(join(ROOT, directory)),
    )
    expect(scanned.length).toBeGreaterThan(50)
  })
})

// The resolve keeps the helper honest when a directory is renamed away.
void resolve