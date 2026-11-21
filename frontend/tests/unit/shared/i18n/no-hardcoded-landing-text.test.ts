import { describe, expect, it } from 'vitest'
import { existsSync, readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const ROOT = process.cwd()

const LANDING_FILES = [
  'app/pages/index.vue',
  'app/pages/how-work.vue',
  'app/pages/about.vue',
  'app/pages/contact.vue',
  'app/pages/terms.vue',
  'app/pages/privacy.vue',
  'app/components/LandingNavbar.vue',
  'app/components/LandingFooter.vue',
  'app/components/PortalFooter.vue',
]

function textNodesWithoutTranslation(content: string): string[] {
  const template = content.split('<script', 1)[0]
  // Code blocks carry source samples, not user-facing copy.
  const withoutCode = template
    .replace(/<pre\b[\s\S]*?<\/pre>/gi, '')
    .replace(/<code\b[\s\S]*?<\/code>/gi, '')

  const offenders: string[] = []
  for (const match of withoutCode.matchAll(/>([^<>]+)</g)) {
    const text = match[1].trim()
    if (!text) continue
    if (!/[A-Za-z]/.test(text)) continue
    if (text.startsWith('{{') || text.startsWith('{%')) continue
    if (text.includes('$t(')) continue
    offenders.push(text)
  }
  return offenders
}

describe('landing copy', () => {
  it('uses i18n for every user-facing text node', () => {
    const offenders = LANDING_FILES.flatMap((file) => {
      const path = resolve(ROOT, file)
      if (!existsSync(path)) return []
      return textNodesWithoutTranslation(readFileSync(path, 'utf-8')).map(
        (text) => `${file}: ${text}`,
      )
    })
    expect(offenders).toEqual([])
  })
})
