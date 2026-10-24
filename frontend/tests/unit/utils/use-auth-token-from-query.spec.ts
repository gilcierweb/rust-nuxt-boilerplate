import { beforeEach, describe, expect, it, vi } from 'vitest'

import { useAuthTokenFromQuery } from '~~/app/utils/use-auth-token-from-query'

/**
 * Behaviour under test: after a page reads its one-time token, the URL no longer
 * carries it, so `router.replace` is called with a path that has no `token`.
 *
 * The route and the router are passed in explicitly, which keeps the test honest
 * about what the composable does with them.
 */
describe('useAuthTokenFromQuery', () => {
  let router: { replace: ReturnType<typeof vi.fn> }

  beforeEach(() => {
    router = { replace: vi.fn() }
  })

  it('exposes the token read from the current query', () => {
    const route = { path: '/auth/reset-password', query: { token: 'a-secret-token' } }

    const { token } = useAuthTokenFromQuery('token', route, router)
    expect(token).toBe('a-secret-token')
  })

  it('replaces the URL with one that no longer carries the token', () => {
    const route = {
      path: '/auth/reset-password',
      query: { token: 'a-secret-token', next: '/portal' },
    }

    const { consume } = useAuthTokenFromQuery('token', route, router)
    consume()

    expect(router.replace).toHaveBeenCalledWith({
      path: '/auth/reset-password?next=%2Fportal',
    })
  })

  it('replaces the URL when the token was the only parameter', () => {
    const route = { path: '/auth/reset-password', query: { token: 'a-secret-token' } }

    const { consume } = useAuthTokenFromQuery('token', route, router)
    consume()

    expect(router.replace).toHaveBeenCalledWith({ path: '/auth/reset-password' })
  })

  it('does not touch the URL when there is no token to remove', () => {
    const route = { path: '/auth/reset-password', query: { next: '/portal' } }

    const { token, consume } = useAuthTokenFromQuery('token', route, router)
    expect(token).toBeNull()

    consume()
    expect(router.replace).not.toHaveBeenCalled()
  })
})
