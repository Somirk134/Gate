import { onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useLogStore } from "../store"

export function useLog() {
  const store = useLogStore()
  const refs = storeToRefs(store)

  onMounted(() => {
    if (store.status === "idle") {
      void store.load()
    }
  })

  return {
    ...refs,
    load: store.load,
    refresh: store.refresh,
    append: store.append,
    appendMany: store.appendMany,
    remove: store.remove,
    clear: store.clear,
    setFilter: store.setFilter,
    resetFilter: store.resetFilter,
    search: store.search,
    select: store.select,
    pause: store.pause,
    resume: store.resume,
    setAutoScroll: store.setAutoScroll,
    generateTestLogs: store.generateTestLogs,
    appendTestLog: store.appendTestLog,
    setLevel: store.setLevel,
    setSource: store.setSource,
    store,
  }
}
