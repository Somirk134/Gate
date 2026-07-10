import { defineStore } from 'pinia'
import { ref, computed, onScopeDispose } from 'vue'
import { createId } from '@/utils/id'

export type DialogType = 'modal' | 'confirm' | 'alert' | 'delete' | 'form'

export interface DialogItem {
  id: string
  type: DialogType
  title: string
  content?: string
  visible: boolean
  props?: Record<string, any>
  resolve?: (value: any) => void
  reject?: (reason?: any) => void
}

export const useDialogStore = defineStore('dialog', () => {
  // === State ===
  const dialogs = ref<DialogItem[]>([])
  const cleanupTimers = new Set<ReturnType<typeof setTimeout>>()

  // === Getters ===
  const activeDialogs = computed(() => dialogs.value.filter((d) => d.visible))
  const hasOpenDialog = computed(() => activeDialogs.value.length > 0)

  // === Actions ===
  function openDialog(options: Omit<DialogItem, 'id' | 'visible'>): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = createId('dialog')
      const dialog: DialogItem = {
        id,
        visible: true,
        ...options,
        resolve,
        reject,
      }
      dialogs.value.push(dialog)
    })
  }

  function closeDialog(id: string, result?: any) {
    const dialog = dialogs.value.find((d) => d.id === id)
    if (dialog) {
      dialog.visible = false
      dialog.resolve?.(result)
      const timer = setTimeout(() => {
        cleanupTimers.delete(timer)
        dialogs.value = dialogs.value.filter((d) => d.id !== id)
      }, 300)
      cleanupTimers.add(timer)
    }
  }

  function dismissDialog(id: string) {
    const dialog = dialogs.value.find((d) => d.id === id)
    if (dialog) {
      dialog.visible = false
      dialog.reject?.('dismissed')
      const timer = setTimeout(() => {
        cleanupTimers.delete(timer)
        dialogs.value = dialogs.value.filter((d) => d.id !== id)
      }, 300)
      cleanupTimers.add(timer)
    }
  }

  function closeAll() {
    dialogs.value.forEach((d) => {
      d.visible = false
      d.reject?.('closed-all')
    })
    const timer = setTimeout(() => {
      cleanupTimers.delete(timer)
      dialogs.value = []
    }, 300)
    cleanupTimers.add(timer)
  }

  onScopeDispose(() => {
    cleanupTimers.forEach(clearTimeout)
    cleanupTimers.clear()
  })

  return {
    dialogs,
    activeDialogs,
    hasOpenDialog,
    openDialog,
    closeDialog,
    dismissDialog,
    closeAll,
  }
})
