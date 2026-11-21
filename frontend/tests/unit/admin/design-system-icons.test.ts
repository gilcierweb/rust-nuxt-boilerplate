import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { extname, join } from 'node:path'

// The design system renders icons through the installed tabler collection
// (`<span class="icon-[tabler--*]">`). An `<Icon>` component referenced by
// live templates was never registered and its collections
// (`heroicons`, `svg-spinners`) are not installed, so every icon on the
// authentication screens silently rendered as nothing.

const ROOT = process.cwd()

const COMPONENT_DIRECTORIES = ['app/components', 'app/pages', 'app/layouts']

function templateFiles(directory: string): string[] {
  if (!existsSync(directory)) return []
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name)
    if (entry.isDirectory()) return templateFiles(path)
    if (extname(entry.name) !== '.vue') return []
    return [path]
  })
}

function missingIconUsages(): Array<{ file: string; line: number }> {
  const usages: Array<{ file: string; line: number }> = []
  for (const directory of COMPONENT_DIRECTORIES) {
    for (const file of templateFiles(join(ROOT, directory))) {
      const lines = readFileSync(file, 'utf-8').split('\n')
      lines.forEach((line, index) => {
        if (/<Icon[\s>]/.test(line)) {
          usages.push({ file: file.slice(ROOT.length + 1), line: index + 1 })
        }
      })
    }
  }
  return usages
}

describe('design system icons', () => {
  it('renders every icon through the tabler convention', () => {
    expect(missingIconUsages()).toEqual([])
  })

  it('verifies the rule scans the templates', () => {
    const scanned = COMPONENT_DIRECTORIES.flatMap((directory) =>
      templateFiles(join(ROOT, directory)),
    )
    expect(scanned.length).toBeGreaterThan(30)
  })
})