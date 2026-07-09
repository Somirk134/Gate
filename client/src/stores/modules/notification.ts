import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type NotificationType = 'success' | 'error' | 'warning' | 'info'

const DEFAULT_NOTIFICATION_DURATION = 4000

export interface NotificationItem {
  id: string
  type: NotificationType
  title: string
  content?: string
  duration?: number
  closable?: boolean
  timestamp: number
}

export const useNotificationStore = defineStore('notification', () => {
  // === State ===
  const notifications = ref<NotificationItem[]>([])
  const maxNotifications = ref(8)

  // === Getters ===
  const activeNotifications = computed(() => notifications.value.slice(0, maxNotifications.value))

  // === Actions ===
  function notify(options: Omit<NotificationItem, 'id' | 'timestamp'>) {
    const id = `notif-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
    const { duration, closable, ...notification } = options
    const notif: NotificationItem = {
      id,
      timestamp: Date.now(),
      duration: duration ?? DEFAULT_NOTIFICATION_DURATION,
      closable: closable ?? true,
      ...notification,
    }
    notifications.value.unshift(notif)

    // Auto dismiss
    if (notif.duration && notif.duration > 0) {
      setTimeout(() => {
        dismiss(id)
      }, notif.duration)
    }

    return id
  }

  function success(title: string, content?: string, duration?: number) {
    return notify({ type: 'success', title, content, duration })
  }

  function error(title: string, content?: string, duration?: number) {
    return notify({ type: 'error', title, content, duration })
  }

  function warning(title: string, content?: string, duration?: number) {
    return notify({ type: 'warning', title, content, duration })
  }

  function info(title: string, content?: string, duration?: number) {
    return notify({ type: 'info', title, content, duration })
  }

  function dismiss(id: string) {
    notifications.value = notifications.value.filter((n) => n.id !== id)
  }

  function clearAll() {
    notifications.value = []
  }

  return {
    notifications,
    activeNotifications,
    notify,
    success,
    error,
    warning,
    info,
    dismiss,
    clearAll,
  }
})
