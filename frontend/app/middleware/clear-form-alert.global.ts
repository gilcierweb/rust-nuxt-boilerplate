export default defineNuxtRouteMiddleware(() => {
  const { clearFormAlertMessage } = useFormAlert()
  clearFormAlertMessage()
})
