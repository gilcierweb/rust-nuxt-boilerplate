import { describe, it, expect, vi } from 'vitest'
import { createApp } from 'vue'

import { adminDeleteButton } from '@/app/utils/admin-ui'

describe('adminDeleteButton', () => {
  it('uses ARRAY children on the native button (array children contract)', () => {
    // A Vue runtime silently drops slot-object children ({ default: () => vnode })
    // with a single VNode on native elements: they are never normalized, so the
    // button renders empty with no dev warning. The helper contract is to always
    // provide ARRAY children so the icon renders both on SSR and client.
    const vnode = adminDeleteButton({ pending: false, onClick: () => {} })

    expect(vnode.type).toBe('button')
    expect(Array.isArray(vnode.children)).toBe(true)
  })

  it('shows the trash icon and stays enabled when not pending', () => {
    const vnode = adminDeleteButton({
      pending: false,
      onClick: () => {},
    })

    expect((vnode.props as { disabled?: boolean }).disabled).toBe(false)

    const child = (vnode.children as unknown[])[0] as {
      type: unknown
      props: { class: string }
    }
    expect(child.type).toBe('span')
    expect(child.props.class).toContain('icon-[tabler--trash]')
  })

  it('shows the spinner icon and disables while the delete is pending', () => {
    const vnode = adminDeleteButton({ pending: true, onClick: () => {} })

    expect((vnode.props as { disabled?: boolean }).disabled).toBe(true)

    const child = (vnode.children as unknown[])[0] as { props: { class: string } }
    expect(child.props.class).toContain('icon-[tabler--loader-2]')
  })

  it('keeps class and type consistent with the other row action buttons', () => {
    const vnode = adminDeleteButton({ pending: false, onClick: () => {} })
    const props = vnode.props as Record<string, unknown>

    expect(props.type).toBe('button')
    expect(props.class).toBe('btn btn-circle btn-text btn-sm text-error')
  })

  it('invokes the provided onClick handler', () => {
    const onClick = vi.fn()
    const vnode = adminDeleteButton({ pending: false, onClick })

    const props = vnode.props as { onClick: () => void }
    props.onClick()

    expect(onClick).toHaveBeenCalledTimes(1)
  })

  it('mounts through the real client renderer with the icon in the DOM', () => {
    // This is the actual bug scenario: without ARRAY children the real renderer
    // produces an empty <button> (silently, no warning).
    const container = document.createElement('div')
    document.body.appendChild(container)

    createApp({
      render: () => adminDeleteButton({ pending: false, onClick: () => {} }),
    }).mount(container)

    expect(container.querySelectorAll('button')).toHaveLength(1)
    expect(container.querySelector('.icon-\\[tabler--trash\\]')).not.toBeNull()

    container.remove()
  })
})
