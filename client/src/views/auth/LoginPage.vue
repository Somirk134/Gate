<template>
  <n-card :title="t('auth.login')" style="max-width: 400px; margin: 100px auto">
    <n-form>
      <n-form-item :label="t('auth.username')">
        <n-input v-model:value="form.username" />
      </n-form-item>
      <n-form-item :label="t('auth.password')">
        <n-input v-model:value="form.password" type="password" />
      </n-form-item>
      <n-form-item>
        <n-button type="primary" block @click="handleLogin">
          {{ t('auth.login') }}
        </n-button>
      </n-form-item>
    </n-form>
  </n-card>
</template>

<script setup lang="ts">
import { reactive } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const form = reactive({
  username: '',
  password: '',
})

function handleLogin() {
  throw new Error('该功能暂未实现：登录后台尚未接入。')
}
</script>
