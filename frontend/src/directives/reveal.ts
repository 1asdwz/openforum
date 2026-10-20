import type { Directive } from 'vue'

const observer =
  typeof window !== 'undefined' && 'IntersectionObserver' in window
    ? new IntersectionObserver(
        (entries) => {
          for (const entry of entries) {
            if (!entry.isIntersecting) continue
            entry.target.classList.add('is-revealed')
            observer?.unobserve(entry.target)
          }
        },
        { rootMargin: '0px 0px -6% 0px', threshold: 0.06 }
      )
    : null

const reduceMotion =
  typeof window !== 'undefined' &&
  window.matchMedia('(prefers-reduced-motion: reduce)').matches

/**
 * v-reveal — entrance animation driven by IntersectionObserver.
 * Optional value is a delay in milliseconds, e.g. `v-reveal="120"`.
 */
export const reveal: Directive<HTMLElement, number | undefined> = {
  mounted(el, binding) {
    if (!observer || reduceMotion) {
      el.classList.add('is-revealed')
      return
    }
    el.classList.add('reveal')
    if (typeof binding.value === 'number') {
      el.style.setProperty('--reveal-delay', `${binding.value}ms`)
    }
    observer.observe(el)
  },
  unmounted(el) {
    observer?.unobserve(el)
  },
}
