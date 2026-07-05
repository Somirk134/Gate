/* ==================================================================
   useCountUp — 数字增长动画
   ------------------------------------------------------------------
   将一个目标数字从 0 平滑增长到目标值，使用 requestAnimationFrame。
   返回响应式的当前显示值，支持 easing。
   ================================================================== */

import { ref, watch, onUnmounted, type Ref } from "vue"

const easeOutQuart = (t: number) => 1 - Math.pow(1 - t, 4)

export function useCountUp(
  target: Ref<number>,
  options: { duration?: number; decimals?: number } = {},
) {
  const { duration = 800, decimals = 0 } = options
  const display = ref(0)
  let raf = 0
  let start = 0
  let from = 0

  function animate(to: number) {
    cancelAnimationFrame(raf)
    from = display.value
    start = performance.now()

    function tick(now: number) {
      const elapsed = now - start
      const progress = Math.min(elapsed / duration, 1)
      const eased = easeOutQuart(progress)
      const current = from + (to - from) * eased
      display.value = Number(current.toFixed(decimals))
      if (progress < 1) {
        raf = requestAnimationFrame(tick)
      } else {
        display.value = to
      }
    }

    raf = requestAnimationFrame(tick)
  }

  watch(
    target,
    (val) => {
      if (val === 0) {
        display.value = 0
        return
      }
      animate(val)
    },
    { immediate: true },
  )

  onUnmounted(() => cancelAnimationFrame(raf))

  return display
}
