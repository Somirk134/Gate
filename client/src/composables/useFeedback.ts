import { useDialogStore, useNotificationStore } from "@stores"

export type ToastType = "success" | "error" | "warning" | "info"

export function useFeedback() {
  const notifications = useNotificationStore()
  const dialogs = useDialogStore()

  const toast = {
    success: (content: string) => notifications.success(content),
    error: (content: string) => notifications.error(content),
    warning: (content: string) => notifications.warning(content),
    info: (content: string) => notifications.info(content),
  }

  const notify = {
    success: (title: string, content?: string) => notifications.success(title, content),
    error: (title: string, content?: string) => notifications.error(title, content),
    warning: (title: string, content?: string) => notifications.warning(title, content),
    info: (title: string, content?: string) => notifications.info(title, content),
  }

  const confirm = (options: {
    title?: string
    content: string
    confirmText?: string
    cancelText?: string
    type?: "info" | "success" | "warning" | "error"
    onConfirm?: () => void | Promise<void>
    onCancel?: () => void
  }) => {
    void dialogs
      .openDialog({
        type: options.type === "error" ? "delete" : "confirm",
        title: options.title ?? "确认操作",
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
      type: "error",
      title: options.title ?? "危险操作",
      content: options.content,
      confirmText: options.confirmText ?? "确认",
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
