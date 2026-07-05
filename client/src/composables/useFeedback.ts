/* ==================================================================
   useFeedback — 通知/对话框/确认 统一 API
   ------------------------------------------------------------------
   用途：封装 Naive UI 的 useMessage / useDialog / useNotification，
   提供项目统一的通知调用入口。所有业务通知必须走本 composable，
   禁止在业务中直接调用 naive 原生 API（便于统一主题/文案/图标）。

   注意：必须在 Naive Provider 的子组件内使用（App.vue 已配置 Provider）。
   ================================================================== */

import { useMessage, useDialog, useNotification } from "naive-ui"

export type ToastType = "success" | "error" | "warning" | "info"

export function useFeedback() {
  const message = useMessage()
  const dialog = useDialog()
  const notification = useNotification()

  /** 轻量 Toast（顶部短时） */
  const toast = {
    success: (content: string) => message.success(content),
    error: (content: string) => message.error(content),
    warning: (content: string) => message.warning(content),
    info: (content: string) => message.info(content),
  }

  /** 确认对话框 */
  const confirm = (options: {
    title?: string
    content: string
    confirmText?: string
    cancelText?: string
    type?: "info" | "success" | "warning" | "error"
    onConfirm?: () => void | Promise<void>
    onCancel?: () => void
  }) => {
    dialog[options.type ?? "warning"]({
      title: options.title ?? "确认操作",
      content: options.content,
      positiveText: options.confirmText ?? "确认",
      negativeText: options.cancelText ?? "取消",
      onPositiveClick: options.onConfirm,
      onNegativeClick: options.onCancel,
    })
  }

  /** 警告确认（危险操作） */
  const confirmDanger = (options: {
    title?: string
    content: string
    confirmText?: string
    onConfirm?: () => void | Promise<void>
  }) => {
    dialog.error({
      title: options.title ?? "危险操作",
      content: options.content,
      positiveText: options.confirmText ?? "确认删除",
      negativeText: "取消",
      onPositiveClick: options.onConfirm,
    })
  }

  /** 通知（右上角，可手动关闭，用于异步结果） */
  const notify = {
    success: (title: string, content?: string) =>
      notification.success({ title, content }),
    error: (title: string, content?: string) =>
      notification.error({ title, content }),
    warning: (title: string, content?: string) =>
      notification.warning({ title, content }),
    info: (title: string, content?: string) =>
      notification.info({ title, content }),
  }

  return { toast, confirm, confirmDanger, notify, message, dialog, notification }
}
