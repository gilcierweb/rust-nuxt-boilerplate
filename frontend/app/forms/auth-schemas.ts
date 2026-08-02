/**
 * Valibot schemas for auth forms.
 *
 * The rules mirror the backend's validator constraints in
 * backend/src/controllers/auth_controller.rs so the user gets fast,
 * localized feedback before the request leaves the browser. Server-side
 * validation still runs - never remove it, this layer is UX only.
 *
 * Conventions:
 *  - All schemas are wrapped in toTypedSchema() at the call site so
 *    vee-validate receives a strongly typed object.
 *  - Error messages are i18n keys resolved at runtime; the literal key
 *    is passed as the valibot message argument. The locale files at
 *    frontend/i18n/locales/{locale}/auth.json carry the actual translations.
 *  - Length bounds come straight from the Rust validator annotations
 *    to keep frontend and backend aligned (e.g. password 12-128).
 */
import * as v from 'valibot'

/** Auth payload contract used by `/auth/login`. */
export type LoginValues = {
  email: string
  password: string
  otp_code: string
}

/**
 * Login form: email + password. OTP is only sent after the first
 * roundtrip when the backend reports `requires_otp: true`.
 */
export const loginSchema = (t: (key: string) => string) =>
  v.object({
    email: v.pipe(
      v.string(),
      v.trim(),
      v.nonEmpty(t('auth.validation.emailRequired')),
      v.email(t('auth.validation.invalidEmail')),
      v.maxLength(254, t('auth.validation.emailTooLong')),
    ),
    password: v.pipe(
      v.string(),
      v.nonEmpty(t('auth.validation.passwordRequired')),
      v.maxLength(128, t('auth.validation.passwordTooLong')),
    ),
    otp_code: v.pipe(
      v.string(),
      v.maxLength(6, t('auth.validation.otpInvalid')),
    ),
  })

/** Auth payload contract used by `/auth/register`. */
export type RegisterValues = {
  email: string
  password: string
  password_confirmation: string
  age_confirmed: boolean
}

/**
 * Register form: email, password (12-128 chars), confirmation match,
 * and a mandatory "I am 18+" terms checkbox.
 *
 * The strength check (length + uppercase + digit + symbol) is intentionally
 * a UX gate, not a security one — the backend runs `validate_password_strength`
 * on top of these rules.
 */
export const registerSchema = (t: (key: string) => string) =>
  v.pipe(
    v.object({
      email: v.pipe(
        v.string(),
        v.trim(),
        v.nonEmpty(t('auth.validation.emailRequired')),
        v.email(t('auth.validation.invalidEmail')),
        v.maxLength(254, t('auth.validation.emailTooLong')),
      ),
      password: v.pipe(
        v.string(),
        v.nonEmpty(t('auth.validation.passwordRequired')),
        v.minLength(12, t('auth.validation.passwordTooShort')),
        v.maxLength(128, t('auth.validation.passwordTooLong')),
        v.regex(/[A-Z]/, t('auth.validation.passwordNeedsUpper')),
        v.regex(/[0-9]/, t('auth.validation.passwordNeedsDigit')),
        v.regex(/[^A-Za-z0-9]/, t('auth.validation.passwordNeedsSymbol')),
      ),
      password_confirmation: v.pipe(
        v.string(),
        v.nonEmpty(t('auth.validation.passwordConfirmRequired')),
        v.maxLength(128, t('auth.validation.passwordTooLong')),
      ),
      age_confirmed: v.pipe(
        v.boolean(),
        v.literal(true, t('auth.validation.termsRequired')),
      ),
    }),
    v.rawCheck(({ dataset, addIssue }) => {
      const obj = dataset.value
      if (
        typeof obj === 'object' && obj !== null &&
        'password' in obj && 'password_confirmation' in obj
      ) {
        const pw = (obj as Record<string, unknown>).password
        const confirm = (obj as Record<string, unknown>).password_confirmation
        if (typeof confirm === 'string' && typeof pw === 'string' && confirm && pw !== confirm) {
          addIssue({
            validation: 'password_mismatch',
            path: [{
              input: obj,
              origin: obj,
              key: 'password_confirmation',
              value: confirm,
              type: 'object',
              schema: v.string(),
            }],
            message: t('auth.validation.passwordMismatch'),
          })
        }
      }
    }),
  )

/** Auth payload contract used by `/auth/forgot-password`. */
export type ForgotPasswordValues = {
  email: string
}

export const forgotPasswordSchema = (t: (key: string) => string) =>
  v.object({
    email: v.pipe(
      v.string(),
      v.trim(),
      v.nonEmpty(t('auth.validation.emailRequired')),
      v.email(t('auth.validation.invalidEmail')),
      v.maxLength(254, t('auth.validation.emailTooLong')),
    ),
  })

/** Auth payload contract used by `/auth/reset`. */
export type ResetPasswordValues = {
  password: string
  password_confirmation: string
}

/**
 * Reset form: same strength rules as register, confirmation must match.
 * The token is taken from the URL query and is not user-editable, so it
 * is not part of the schema.
 */
export const resetPasswordSchema = (t: (key: string) => string) =>
  v.pipe(
    v.object({
      password: v.pipe(
        v.string(),
        v.nonEmpty(t('auth.validation.passwordRequired')),
        v.minLength(12, t('auth.validation.passwordTooShort')),
        v.maxLength(128, t('auth.validation.passwordTooLong')),
        v.regex(/[A-Z]/, t('auth.validation.passwordNeedsUpper')),
        v.regex(/[0-9]/, t('auth.validation.passwordNeedsDigit')),
        v.regex(/[^A-Za-z0-9]/, t('auth.validation.passwordNeedsSymbol')),
      ),
      password_confirmation: v.pipe(
        v.string(),
        v.nonEmpty(t('auth.validation.passwordConfirmRequired')),
        v.maxLength(128, t('auth.validation.passwordTooLong')),
      ),
    }),
    v.rawCheck(({ dataset, addIssue }) => {
      const obj = dataset.value
      if (
        typeof obj === 'object' && obj !== null &&
        'password' in obj && 'password_confirmation' in obj
      ) {
        const pw = (obj as Record<string, unknown>).password
        const confirm = (obj as Record<string, unknown>).password_confirmation
        if (typeof confirm === 'string' && typeof pw === 'string' && confirm && pw !== confirm) {
          addIssue({
            validation: 'password_mismatch',
            path: [{
              input: obj,
              origin: obj,
              key: 'password_confirmation',
              value: confirm,
              type: 'object',
              schema: v.string(),
            }],
            message: t('auth.validation.passwordMismatch'),
          })
        }
      }
    }),
  )
