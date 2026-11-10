import { readonly, ref } from 'vue'

// Vendor-neutral CAPTCHA hand-off.
//
// The backend only ever sees an opaque token string (`captcha_token`), so no
// CAPTCHA vendor is imposed on this project. A deployment that wants human
// verification renders the widget of its choice (Turnstile, reCAPTCHA,
// hCaptcha, a self-hosted service, ...) and hands the solved token over with
// `setToken()`. Without a widget the token stays `null` and registration
// behaves exactly as before.
//
// The token is only ever produced client-side (a widget runs in the browser),
// so the shared module state never carries request data between SSR renders.
const sharedToken = ref<string | null>(null)

export function useCaptchaToken() {
  return {
    token: readonly(sharedToken),
    /** Hand the solved token over to the form. Blank values clear it. */
    setToken(value: string) {
      sharedToken.value = value.trim() || null
    },
    /** Forget the current token (after a failed submission, for instance). */
    clear() {
      sharedToken.value = null
    },
  }
}
