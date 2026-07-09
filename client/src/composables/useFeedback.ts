import { useI18n } from 'vue-i18n'
import { useDialogStore, useNotificationStore } from '@stores'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export function useFeedback() {
  const notifications = useNotificationStore()
  const dialogs = useDialogStore()
  const { t } = useI18n()

  const toast = {
    success: (content: string) => notifications.success(content),
    error: (content: string) => notifications.error(content),
    warning: (content: string) => notifications.warning(content),
    info: (content: string) => notifications.info(content),
  }

  const notify = {
    success: (title: string, content?: string, duration?: number) =>
      notifications.success(title, content, duration),
    error: (title: string, content?: string, duration?: number) =>
      notifications.error(title, content, duration),
    warning: (title: string, content?: string, duration?: number) =>
      notifications.warning(title, content, duration),
    info: (title: string, content?: string, duration?: number) =>
      notifications.info(title, content, duration),
  }

  const confirm = (options: {
    title?: string
    content: string
    confirmText?: string
    cancelText?: string
    type?: 'info' | 'success' | 'warning' | 'error'
    onConfirm?: () => void | Promise<void>
    onCancel?: () => void
  }) => {
    void dialogs
      .openDialog({
        type: options.type === 'error' ? 'delete' : 'confirm',
        title: options.title ?? t('dialog.confirmOperation'),
        content: options.content,
        props: {
          confirmText: options.confirmText,
          cancelText: options.cancelText,
        },
      })
      .then(async () => {
        await options.onConfirm?.()
      })
      .catch(() => {
        options.onCancel?.()
      })
  }

  const confirmDanger = (options: {
    title?: string
    content: string
    confirmText?: string
    onConfirm?: () => void | Promise<void>
  }) => {
    confirm({
      type: 'error',
      title: options.title ?? t('dialog.dangerOperation'),
      content: options.content,
      // 默认危险确认按钮交给 i18n，避免不同弹窗入口各自硬编码。
      confirmText: options.confirmText ?? t('dialog.confirm'),
      onConfirm: options.onConfirm,
    })
  }

  return {
    toast,
    notify,
    confirm,
    confirmDanger,
    message: toast,
    dialog: dialogs,
    notification: notifications,
  }
}
