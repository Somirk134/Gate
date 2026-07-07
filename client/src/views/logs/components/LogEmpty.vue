<template>
  <div class="log-empty">
    <div class="log-empty__icon">
      <GIcon name="terminal" :size="44" />
    </div>
    <h2>Log Center</h2>
    <p>
      暂无运行时日志。连接后端、创建 Tunnel 或执行真实操作后，Tracing 与 Runtime 日志会显示在这里。
    </p>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
</script>
