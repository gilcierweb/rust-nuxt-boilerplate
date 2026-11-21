import { readonly } from 'vue'

// Vendor-neutral CAPTCHA hand-off.
//
// The backend only ever sees an opaque token string (`captcha_token`), so no
// CAPTCHA vendor is imposed on this project. A deployment that wants human
// verification renders the widget of its choice (Turnstile, reCAPTCHA,
// hCaptcha, a self-hosted service, ...) and hands the solved token over with
// `setToken()`. Without a widget the token stays null and registration
// behaves exactly as before.
//
// The token is only ever produced client-side (a widget runs in the browser),
// but the sharing must be too. A module-scope ref is shared by every
// concurrent SSR render of the process, so one request could write the token
// of another; `useState` gives per-request state on the server and a
// singleton on the client, the same contract the project documents in
// `useToast`.
export function useCaptchaToken() {
  const state = useState<string | null>('captchaToken', () => null)

  return {
    token: readonly(state),
    /** Hand the solved token over to the form. Blank values clear it. */
    setToken(value: string) {
      state.value = value.trim() || null
    },
    /** Forget the current token (after a failed submission, for instance). */
    clear() {
      state.value = null
    },
  }
}
