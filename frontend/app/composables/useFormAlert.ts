export const useFormAlert = () => {
  const formAlertMessage = useState<string>('admin-form-alert-message', () => '')

  const setFormAlertMessage = (message: string) => {
    formAlertMessage.value = message
  }

  const clearFormAlertMessage = () => {
    formAlertMessage.value = ''
  }

  return {
    formAlertMessage,
    setFormAlertMessage,
    clearFormAlertMessage,
  }
}
