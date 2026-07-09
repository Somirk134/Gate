import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export const useAppStore = defineStore('app', () => {
  const theme = ref<'light' | 'dark'>('light')
  const language = ref('zh-CN')
  const connected = ref(false)
  const serverAddr = ref('')

  const isConnected = computed(() => connected.value)

  function setTheme(t: 'light' | 'dark') {
    theme.value = t
  }

  function setConnected(status: boolean) {
    connected.value = status
  }

  return {
    theme,
    language,
    connected,
    serverAddr,
    isConnected,
    setTheme,
    setConnected,
  }
})
