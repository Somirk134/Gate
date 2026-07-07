import { defineStore } from "pinia"
import { computed, ref } from "vue"
import type { Project, ProjectFormData, ProjectLoadStatus } from "../types"

export const useProjectStore = defineStore("project", () => {
  const projects = ref<Project[]>([])
  const status = ref<ProjectLoadStatus>("idle")
  const error = ref<string>("")
  const lastUpdated = ref<number>(0)
  const serverNames = ref<string[]>([])

  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")
  const hasProjects = computed(() => projects.value.length > 0)
  const pinnedProjects = computed(() => projects.value.filter((p) => p.pinned))
  const favoriteProjects = computed(() => projects.value.filter((p) => p.favorite))
  const runningProjects = computed(() => projects.value.filter((p) => p.status !== "stopped"))
  const stoppedProjects = computed(() => projects.value.filter((p) => p.status === "stopped"))
  const recentProjects = computed(() => projects.value.slice(0, 8))
  const totalTunnels = computed(() => 0)
  const runningTunnelCount = computed(() => 0)

  function getById(id: string): Project | undefined {
    return projects.value.find((p) => p.id === id)
  }

  async function load(): Promise<void> {
    status.value = "success"
    error.value = ""
    projects.value = []
    serverNames.value = []
    lastUpdated.value = Date.now()
  }

  async function refresh(): Promise<void> {
    return load()
  }

  function notImplemented(): never {
    error.value = "该功能暂未实现"
    throw new Error(error.value)
  }

  function createProject(_form: ProjectFormData): Project {
    return notImplemented()
  }

  function updateProject(_id: string, _patch: Partial<ProjectFormData>): void {
    notImplemented()
  }

  function removeProject(_id: string): void {
    notImplemented()
  }

  function startProject(_id: string): void {
    notImplemented()
  }

  function stopProject(_id: string): void {
    notImplemented()
  }

  function togglePin(_id: string): void {
    notImplemented()
  }

  function toggleFavorite(_id: string): void {
    notImplemented()
  }

  return {
    projects,
    status,
    error,
    lastUpdated,
    serverNames,
    isLoading,
    isError,
    isReady,
    hasProjects,
    pinnedProjects,
    favoriteProjects,
    runningProjects,
    stoppedProjects,
    recentProjects,
    totalTunnels,
    runningTunnelCount,
    getById,
    load,
    refresh,
    createProject,
    updateProject,
    removeProject,
    startProject,
    stopProject,
    togglePin,
    toggleFavorite,
  }
})
