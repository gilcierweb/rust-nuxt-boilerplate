import { describe, it, expect } from 'vitest'

import { useCaptchaToken } from '../../../app/composables/useCaptchaToken'

// ---------------------------------------------------------------------------
// useCaptchaToken — vendor-neutral CAPTCHA token hand-off
// ---------------------------------------------------------------------------

describe('useCaptchaToken', () => {
  it('starts without a token', () => {
    const { token, clear } = useCaptchaToken()
    clear()

    expect(token.value).toBeNull()
  })

  it('stores the token handed over by the widget', () => {
    const { token, setToken, clear } = useCaptchaToken()
    clear()

    setToken('solved-token')

    expect(token.value).toBe('solved-token')
  })

  it('trims the token and treats a blank one as absent', () => {
    const { token, setToken, clear } = useCaptchaToken()
    clear()

    setToken('  padded-token  ')
    expect(token.value).toBe('padded-token')

    setToken('   ')
    expect(token.value).toBeNull()
  })

  it('shares the token between the widget and the form', () => {
    const widget = useCaptchaToken()
    const form = useCaptchaToken()
    form.clear()

    widget.setToken('shared-token')

    expect(form.token.value).toBe('shared-token')
    form.clear()
  })
})
