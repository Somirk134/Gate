/* ==================================================================
   useDashboardClock — 响应式时钟
   ------------------------------------------------------------------
   提供实时更新的当前时间，供 WelcomeCard 等组件使用。
   自动清理定时器。
   ================================================================== */

import { ref, onMounted, onUnmounted, computed } from "vue"

export function useDashboardClock() {
  const now = ref(Date.now())
  let timer = 0

  onMounted(() => {
    timer = window.setInterval(() => {
      now.value = Date.now()
    }, 1000)
  })

  onUnmounted(() => {
    clearInterval(timer)
  })

  const date = computed(() => new Date(now.value))

  const timeText = computed(() => {
    const d = date.value
    const h = String(d.getHours()).padStart(2, "0")
    const m = String(d.getMinutes()).padStart(2, "0")
    const s = String(d.getSeconds()).padStart(2, "0")
    return `${h}:${m}:${s}`
  })

  const dateText = computed(() => {
    const d = date.value
    const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"]
    return `${d.getFullYear()}年${d.getMonth() + 1}月${d.getDate()}日 ${weekdays[d.getDay()]}`
  })

  const greeting = computed(() => {
    const h = date.value.getHours()
    if (h < 6) return "夜深了"
    if (h < 12) return "早上好"
    if (h < 14) return "中午好"
    if (h < 18) return "下午好"
    return "晚上好"
  })

  return { now, date, timeText, dateText, greeting }
}
