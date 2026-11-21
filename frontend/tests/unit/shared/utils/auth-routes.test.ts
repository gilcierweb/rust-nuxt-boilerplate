import { describe, expect, it } from 'vitest'

import { isAdminPath } from '~/utils/auth-routes'

// The admin guard must classify an admin route whatever locale prefix the
// router puts in front of it. A hand-written allowlist of prefixes drifts the
// moment a locale is added: the `pt` alias was registered and the guard kept
// checking only `en` and `es`, so `/pt/admin/**` opened to a non admin.

describe('isAdminPath', () => {
  it('recognizes the default locale path', () => {
    expect(isAdminPath('/admin')).toBe(true)
    expect(isAdminPath('/admin/users')).toBe(true)
    expect(isAdminPath('/admin/users/123/edit')).toBe(true)
  })

  it('recognizes every locale prefix the app registers', () => {
    expect(isAdminPath('/en/admin')).toBe(true)
    expect(isAdminPath('/es/admin/users')).toBe(true)
    expect(isAdminPath('/pt/admin/users')).toBe(true)
    expect(isAdminPath('/pt-BR/admin/roles')).toBe(true)
  })

  it('does not classify a lookalike path as admin', () => {
    expect(isAdminPath('/administrator')).toBe(false)
    expect(isAdminPath('/en/administrators')).toBe(false)
    expect(isAdminPath('/portal/dashboard')).toBe(false)
    expect(isAdminPath('/pt/portal')).toBe(false)
    expect(isAdminPath('/')).toBe(false)
  })
})