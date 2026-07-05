<!--
  ServerEmpty — 空状态 / 欢迎页
  ------------------------------------------------------------------
  没有服务器时显示欢迎页面，引导添加第一个服务器。
-->
<template>
  <div class="server-empty">
    <div class="server-empty__illustration">
      <GIcon name="servers" :size="48" />
    </div>
    <h2 class="server-empty__title">Server Cluster</h2>
    <p class="server-empty__desc">
      添加你的第一台 Gate 服务器（个人服务器 / 云服务器 / NAS / Docker），
      统一管理所有 Tunnel 资源。
    </p>
    <p class="server-empty__hint">
      <GIcon name="info-circle" :size="12" />
      所有 Tunnel 必须绑定一个 Server
    </p>
    <div class="server-empty__action">
      <GButton variant="primary" icon="plus" @click="$emit('create')">
        Add Server
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"

defineEmits<{ create: [] }>()
</script>
