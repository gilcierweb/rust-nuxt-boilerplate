import { describe, expect, it } from 'vitest'

import { useEditPath, useShowPath } from '@/app/utils/admin-resource-helpers'

describe('admin-resource-helpers', () => {
  it('builds the show path', () => {
    expect(useShowPath('roles', '123')).toBe('/admin/roles/123')
  })

  it('builds the edit path', () => {
    expect(useEditPath('roles', '123')).toBe('/admin/roles/123/edit')
  })
})
