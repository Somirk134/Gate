<template>
    <n-space vertical>
        <n-h1>Settings</n-h1>
        <n-form label-placement="left">
            <n-form-item label="Server Address">
                <n-input v-model:value="config.serverAddr" placeholder="localhost:5800" />
            </n-form-item>
            <n-form-item label="Auth Token">
                <n-input v-model:value="config.authToken" type="password" />
            </n-form-item>
            <n-form-item label="Auto Connect">
                <n-switch v-model:value="config.autoConnect" />
            </n-form-item>
            <n-form-item>
                <n-button type="primary" @click="saveConfig">Save</n-button>
            </n-form-item>
        </n-form>
    </n-space>
</template>

<script setup lang="ts">
import { reactive } from 'vue'

const config = reactive({
    serverAddr: '',
    authToken: '',
    autoConnect: false,
})

function saveConfig() {
    todo
}
</script>
