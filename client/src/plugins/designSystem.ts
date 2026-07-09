/* ==================================================================
   designSystemPlugin — 全局注册设计系统组件
   ------------------------------------------------------------------
   用途：将 Design System 的基础/反馈/布局/卡片/状态/业务组件全局注册，
   使任意页面/组件可直接使用 <GButton /> <GIcon /> 等，无需逐个 import。
   图标 GIcon 同样全局可用。

   仅注册"无副作用、通用"的组件；带强业务逻辑的组件不在此注册。
   ================================================================== */

import type { App, Plugin } from 'vue'

import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GCard from '@components/base/GCard.vue'
import GBadge from '@components/base/GBadge.vue'

import GInput from '@components/form/GInput.vue'
import GPasswordInput from '@components/form/GPasswordInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import GNumberInput from '@components/form/GNumberInput.vue'
import GSearchInput from '@components/form/GSearchInput.vue'
import GPortInput from '@components/form/GPortInput.vue'
import GHostInput from '@components/form/GHostInput.vue'
import GTokenInput from '@components/form/GTokenInput.vue'
import GFormField from '@components/form/GFormField.vue'
import GLabel from '@components/form/GLabel.vue'

import GSpinner from '@components/feedback/GSpinner.vue'
import GSkeleton from '@components/feedback/GSkeleton.vue'
import GProgress from '@components/feedback/GProgress.vue'
import GCircleProgress from '@components/feedback/GCircleProgress.vue'
import GEmptyState from '@components/feedback/GEmptyState.vue'
import GErrorState from '@components/feedback/GErrorState.vue'

import GStatusDot from '@components/status/GStatusDot.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'

import GPageContainer from '@components/layout/GPageContainer.vue'
import GPageHeader from '@components/layout/GPageHeader.vue'
import GSectionHeader from '@components/layout/GSectionHeader.vue'

import GStatCard from '@components/cards/GStatCard.vue'
import GActionCard from '@components/cards/GActionCard.vue'

/** 全局组件清单：组件名 → 组件 */
const globalComponents = {
  GIcon,
  GButton,
  GIconButton,
  GCard,
  GBadge,
  GInput,
  GPasswordInput,
  GTextarea,
  GNumberInput,
  GSearchInput,
  GPortInput,
  GHostInput,
  GTokenInput,
  GFormField,
  GLabel,
  GSpinner,
  GSkeleton,
  GProgress,
  GCircleProgress,
  GEmptyState,
  GErrorState,
  GStatusDot,
  GStatusBadge,
  GPageContainer,
  GPageHeader,
  GSectionHeader,
  GStatCard,
  GActionCard,
}

export const designSystemPlugin: Plugin = {
  install(app: App) {
    for (const [name, component] of Object.entries(globalComponents)) {
      app.component(name, component)
    }
  },
}
