/* ==================================================================
   useScrollReveal — 滚动入场动画指令
   ------------------------------------------------------------------
   基于 IntersectionObserver 的滚动揭示动画。
   使用方式：在元素上添加 v-scroll-reveal 指令，元素进入视口时
   自动添加 .is-revealed 类触发 CSS 过渡。

   也可作为函数调用，对指定根容器的子元素批量注册。
   ================================================================== */

import { onMounted, onUnmounted, type Directive } from 'vue'

export const vScrollReveal: Directive<HTMLElement> = {
  mounted(el) {
    el.classList.add('scroll-reveal')
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('is-revealed')
            observer.unobserve(entry.target)
          }
        })
      },
      { threshold: 0.08, rootMargin: '0px 0px -40px 0px' },
    )
    observer.observe(el)
    // 存储 observer 以便卸载
    ;(el as any).__revealObserver = observer
  },
  unmounted(el) {
    const observer = (el as any).__revealObserver as IntersectionObserver | undefined
    if (observer) observer.disconnect()
  },
}

/** 批量注册容器内所有 .scroll-reveal 元素 */
export function useScrollReveal() {
  let observer: IntersectionObserver | null = null

  function observe(root: HTMLElement) {
    observer?.disconnect()
    observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            entry.target.classList.add('is-revealed')
            observer?.unobserve(entry.target)
          }
        })
      },
      { threshold: 0.08, rootMargin: '0px 0px -40px 0px' },
    )
    const targets = root.querySelectorAll('.scroll-reveal')
    targets.forEach((t) => observer?.observe(t))
  }

  function disconnect() {
    observer?.disconnect()
    observer = null
  }

  onMounted(() => {})
  onUnmounted(disconnect)

  return { observe, disconnect }
}
