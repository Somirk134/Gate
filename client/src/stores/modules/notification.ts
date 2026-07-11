import { defineStore } from 'pinia'
import { ref, computed, onScopeDispose } from 'vue'
import { createId } from '@/utils/id'

export type NotificationType = 'success' | 'error' | 'warning' | 'info'

const DEFAULT_NOTIFICATION_DURATION = 4000
const MAX_HISTORY = 50

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
  /** 持久化通知历史 — 不自动消失，用于铃铛弹窗展示 */
  const history = ref<NotificationItem[]>([])
  const detailItem = ref<NotificationItem | null>(null)
  const maxNotifications = ref(8)
  const dismissTimers = new Map<string, ReturnType<typeof setTimeout>>()

  // === Getters ===
  const activeNotifications = computed(() => notifications.value.slice(0, maxNotifications.value))
  /** 最近的历史记录（倒序，最新的在前） */
  const recentHistory = computed(() => history.value.slice(0, 20))

  // === Actions ===
  function notify(options: Omit<NotificationItem, 'id' | 'timestamp'> & { persist?: boolean }) {
    const id = createId('notif')
    const { duration, closable, persist, ...rest } = options
    const notif: NotificationItem = {
      id,
      timestamp: Date.now(),
      duration: duration ?? DEFAULT_NOTIFICATION_DURATION,
      closable: closable ?? true,
      ...rest,
    }
    notifications.value.unshift(notif)

    // 自动加入历史（持久记录）
    if (options.persist === undefined || options.persist) {
      pushHistory(notif)
    }

    // Auto dismiss — 仅对 toast 生效
    if (notif.duration && notif.duration > 0) {
      const timer = setTimeout(() => {
        dismissTimers.delete(id)
        dismiss(id)
      }, notif.duration)
      dismissTimers.set(id, timer)
    }

    return id
  }

  /**
   * 推送一条持久化通知到历史记录（不自动消失）。
   * 用于隧道创建/删除、服务器连接等关键事件。
   */
  function pushHistory(item: Omit<NotificationItem, 'id' | 'timestamp' | 'duration'>) {
    const id = createId('hist')
    const entry: NotificationItem = {
      id,
      duration: 0, // 不自动消失
      closable: true,
      timestamp: Date.now(),
      ...item,
    }
    history.value.unshift(entry)
    // 超过上限时裁剪尾部
    if (history.value.length > MAX_HISTORY) {
      history.value = history.value.slice(0, MAX_HISTORY)
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
    const timer = dismissTimers.get(id)
    if (timer) {
      clearTimeout(timer)
      dismissTimers.delete(id)
    }
    notifications.value = notifications.value.filter((n) => n.id !== id)
  }

  onScopeDispose(() => {
    dismissTimers.forEach(clearTimeout)
    dismissTimers.clear()
  })

  function dismissHistory(id: string) {
    history.value = history.value.filter((n) => n.id !== id)
  }

  function clearAll() {
    notifications.value = []
  }

  function clearHistory() {
    history.value = []
  }

  function showDetail(item: NotificationItem) {
    detailItem.value = item
  }

  function closeDetail() {
    detailItem.value = null
  }

  return {
    notifications,
    history,
    detailItem,
    activeNotifications,
    recentHistory,
    notify,
    pushHistory,
    success,
    error,
    warning,
    info,
    dismiss,
    dismissHistory,
    clearAll,
    clearHistory,
    showDetail,
    closeDetail,
  }
})
