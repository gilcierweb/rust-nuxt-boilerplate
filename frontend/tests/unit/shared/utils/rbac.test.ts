import { describe, expect, it } from 'vitest'

import { effectLabel, splitCode } from '@/app/utils/rbac'

describe('splitCode', () => {
  it('splits a resource and an action', () => {
    expect(splitCode('users:read')).toEqual({ resource: 'users', action: 'read' })
    expect(splitCode('roles:update')).toEqual({ resource: 'roles', action: 'update' })
  })

  it('refuses a code without the separator instead of guessing', () => {
    expect(splitCode('users')).toBeNull()
  })

  it('refuses an empty side', () => {
    expect(splitCode(':read')).toBeNull()
    expect(splitCode('users:')).toBeNull()
    expect(splitCode('')).toBeNull()
  })

  it('refuses a code with more than one separator', () => {
    // A second colon would make the split ambiguous, so the helper has no
    // honest answer for it.
    expect(splitCode('users:read:extra')).toBeNull()
  })
})

describe('effectLabel', () => {
  it('maps the allow effect to its i18n key', () => {
    expect(effectLabel(0)).toBe('admin.permissions.effect.allow')
  })

  it('maps the deny effect to its i18n key', () => {
    expect(effectLabel(1)).toBe('admin.permissions.effect.deny')
  })

  it('has no label for a value outside the enum', () => {
    expect(effectLabel(2 as never)).toBeNull()
    expect(effectLabel(undefined as never)).toBeNull()
  })
})
