<template>
    <n-card title="Register" style="max-width: 400px; margin: 100px auto">
        <n-form>
            <n-form-item label="Username">
                <n-input v-model:value="form.username" />
            </n-form-item>
            <n-form-item label="Email">
                <n-input v-model:value="form.email" />
            </n-form-item>
            <n-form-item label="Password">
                <n-input v-model:value="form.password" type="password" />
            </n-form-item>
            <n-form-item>
                <n-button type="primary" block @click="handleRegister">Register</n-button>
            </n-form-item>
        </n-form>
    </n-card>
</template>

<script setup lang="ts">
import { reactive } from 'vue'

const form = reactive({
    username: '',
    email: '',
    password: '',
})

function handleRegister() {
    todo
}
</script>
