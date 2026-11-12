import type { PermissionEffect } from '~/types/rbac'

/** The resource and the action of a permission code. */
export type PermissionCodeParts = {
  resource: string
  action: string
}

export type EffectLabelKey =
  | 'admin.permissions.effect.allow'
  | 'admin.permissions.effect.deny'

/**
 * Splits a `resource:action` permission code.
 *
 * A code with no separator, an empty side, or more than one separator has no
 * honest split, so the helper refuses it instead of guessing where the resource
 * ends.
 */
export function splitCode(code: string): PermissionCodeParts | null {
  const parts = code.split(':')
  if (parts.length !== 2) return null

  const [resource, action] = parts
  if (!resource || !action) return null

  return { resource, action }
}

/** The i18n key of the label of a permission effect. */
export function effectLabel(effect: PermissionEffect): EffectLabelKey | null {
  if (effect === 0) return 'admin.permissions.effect.allow'
  if (effect === 1) return 'admin.permissions.effect.deny'
  return null
}
