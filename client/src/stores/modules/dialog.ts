import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

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

    // === Getters ===
    const activeDialogs = computed(() => dialogs.value.filter(d => d.visible))
    const hasOpenDialog = computed(() => activeDialogs.value.length > 0)

    // === Actions ===
    function openDialog(options: Omit<DialogItem, 'id' | 'visible'>): Promise<any> {
        return new Promise((resolve, reject) => {
            const id = `dialog-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
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
        const dialog = dialogs.value.find(d => d.id === id)
        if (dialog) {
            dialog.visible = false
            dialog.resolve?.(result)
            setTimeout(() => {
                dialogs.value = dialogs.value.filter(d => d.id !== id)
            }, 300)
        }
    }

    function dismissDialog(id: string) {
        const dialog = dialogs.value.find(d => d.id === id)
        if (dialog) {
            dialog.visible = false
            dialog.reject?.('dismissed')
            setTimeout(() => {
                dialogs.value = dialogs.value.filter(d => d.id !== id)
            }, 300)
        }
    }

    function closeAll() {
        dialogs.value.forEach(d => {
            d.visible = false
            d.reject?.('closed-all')
        })
        setTimeout(() => {
            dialogs.value = []
        }, 300)
    }

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
