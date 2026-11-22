import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'

const LOCALES = ['en', 'es', 'pt-BR']

describe('landing copyright', () => {
  it('keeps the year in the template and the boilerplate identity in the message', () => {
    for (const locale of LOCALES) {
      const path = resolve(process.cwd(), `i18n/locales/${locale}/landing.json`)
      const { landing } = JSON.parse(readFileSync(path, 'utf-8'))
      const message: string = landing.footer.copyright

      expect(message, `${locale}: must not repeat the copyright symbol`).not.toContain('©')
      expect(message, `${locale}: must not freeze the year`).not.toMatch(/\b(?:19|20)\d{2}\b/)
      expect(message, `${locale}: must name the boilerplate`).toContain('RustNuxt Boilerplate')
      expect(message, `${locale}: must state the license`).toContain('MIT')
    }
  })
})
