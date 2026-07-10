import { computed } from 'vue'
import {
  useLayoutStore,
  useNavigationStore,
  useThemeStore,
  useDialogStore,
  useNotificationStore,
  useLoadingStore,
} from '@stores'

/* ==================================================================
   useShell — Shell 统一操作 Composable
   聚合所有 Shell 相关操作，业务层只需使用此 hook 即可操作框架
   ================================================================== */

export function useShell() {
  const layout = useLayoutStore()
  const navigation = useNavigationStore()
  const theme = useThemeStore()
  const dialog = useDialogStore()
  const notification = useNotificationStore()
  const loading = useLoadingStore()

  const isDark = computed(() => theme.isDark)

  return {
    // Layout
    layout,
    toggleSidebar: layout.toggleSidebar,
    openCommandPalette: layout.openCommandPalette,
    closeCommandPalette: layout.closeCommandPalette,

    // Navigation
    navigation,
    setBreadcrumbs: navigation.setBreadcrumbs,

    // Theme
    theme,
    toggleTheme: theme.toggleTheme,
    isDark,

    // Dialog
    dialog,
    openDialog: dialog.openDialog,
    closeDialog: dialog.closeDialog,
    closeAllDialogs: dialog.closeAll,

    // Notification
    notification,
    toast: notification,

    // Loading
    loading,
    startLoading: loading.startLoading,
    stopLoading: loading.stopLoading,
  }
}
