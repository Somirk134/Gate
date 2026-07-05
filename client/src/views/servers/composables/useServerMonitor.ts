/* ==================================================================
   useServerMonitor — 服务器实时监控组合式函数
   ------------------------------------------------------------------
   定时驱动 store.tick()，模拟实时资源 / 流量 / 日志更新。
   组件挂载时自动启动，卸载时自动停止。
   未来替换为真实 Rust Server 指标时，改为订阅 WebSocket / IPC 推送即可。
   ================================================================== */

import { onMounted, onBeforeUnmount, ref } from "vue"
import type { useServerStore } from "../store/server"

export function useServerMonitor(
  store: ReturnType<typeof useServerStore>,
  intervalMs = 1000,
) {
  const active = ref(false)
  let timer: ReturnType<typeof setInterval> | null = null

  function start() {
    if (active.value) return
    active.value = true
    timer = setInterval(() => {
      store.tick()
    }, intervalMs)
  }

  function stop() {
    active.value = false
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  function setInterval_(ms: number) {
    const wasActive = active.value
    stop()
    intervalMs = ms
    if (wasActive) start()
  }

  onMounted(() => {
    start()
  })

  onBeforeUnmount(() => {
    stop()
  })

  return { active, start, stop, setInterval: setInterval_ }
}
