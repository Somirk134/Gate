import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface LoadingTask {
    id: string
    message: string
    progress?: number
    indeterminate?: boolean
}

export const useLoadingStore = defineStore('loading', () => {
    // === State ===
    const tasks = ref<LoadingTask[]>([])
    const globalMessage = ref('')
    const globalVisible = ref(false)

    // === Getters ===
    const isLoading = computed(() => tasks.value.length > 0 || globalVisible.value)
    const currentTask = computed(() => tasks.value[0] || null)
    const taskCount = computed(() => tasks.value.length)

    // === Actions ===
    function startLoading(message: string = 'Loading...') {
        globalMessage.value = message
        globalVisible.value = true
    }

    function stopLoading() {
        globalVisible.value = false
        globalMessage.value = ''
    }

    function addTask(message: string, id?: string): string {
        const taskId = id || `loading-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
        tasks.value.push({
            id: taskId,
            message,
            indeterminate: true,
        })
        return taskId
    }

    function updateTask(id: string, progress: number) {
        const task = tasks.value.find(t => t.id === id)
        if (task) {
            task.progress = progress
            task.indeterminate = false
        }
    }

    function removeTask(id: string) {
        tasks.value = tasks.value.filter(t => t.id !== id)
    }

    function clearAll() {
        tasks.value = []
        globalVisible.value = false
    }

    return {
        tasks,
        globalMessage,
        globalVisible,
        isLoading,
        currentTask,
        taskCount,
        startLoading,
        stopLoading,
        addTask,
        updateTask,
        removeTask,
        clearAll,
    }
})
