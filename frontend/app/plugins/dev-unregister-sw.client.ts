export default defineNuxtPlugin(() => {
  if (!import.meta.dev || !('serviceWorker' in navigator)) {
    return
  }

  window.addEventListener('load', async () => {
    const registrations = await navigator.serviceWorker.getRegistrations()

    await Promise.all(registrations.map((registration) => registration.unregister()))

    if ('caches' in window) {
      const cacheKeys = await caches.keys()
      await Promise.all(cacheKeys.map((key) => caches.delete(key)))
    }
  })
})
