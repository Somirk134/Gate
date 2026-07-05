<!--
  TunnelEmpty — 空状态 / 欢迎页
  ------------------------------------------------------------------
  没有隧道时显示欢迎页面，引导创建第一个隧道。
-->
<template>
  <div class="tunnel-empty">
    <div class="tunnel-empty__illustration">
      <GIcon name="router" :size="48" />
    </div>
    <h2 class="tunnel-empty__title">Zero Config 隧道</h2>
    <p class="tunnel-empty__desc">
      点击「New Tunnel」，填写本地地址，点击 Start —— 公网即可访问。
    </p>
    <p class="tunnel-empty__hint">
      <GIcon name="info-circle" :size="12" />
      无需理解 FRP、配置文件或反向代理
    </p>
    <div class="tunnel-empty__action">
      <GButton variant="primary" icon="plus" @click="$emit('create')">
        New Tunnel
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"

defineEmits<{ create: [] }>()
</script>
