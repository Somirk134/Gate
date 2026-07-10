import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import { resolve, dirname } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
    define: {
        // Tauri CSP 禁止 unsafe-eval，开启 vue-i18n JIT 以避免运行时 new Function 编译翻译文案。
        __INTLIFY_JIT_COMPILATION__: 'true',
        __INTLIFY_DROP_MESSAGE_COMPILER__: 'false',
        __INTLIFY_PROD_DEVTOOLS__: 'false',
    },
    plugins: [
        vue(),
        AutoImport({
            imports: [
                'vue',
                'vue-router',
                'pinia',
                '@vueuse/core',
            ],
            dts: 'src/types/auto-imports.d.ts',
        }),
        Components({
            resolvers: [NaiveUiResolver()],
            dts: 'src/types/components.d.ts',
        }),
    ],
    resolve: {
        alias: {
            '@': resolve(__dirname, 'src'),
            '@views': resolve(__dirname, 'src/views'),
            '@components': resolve(__dirname, 'src/components'),
            '@composables': resolve(__dirname, 'src/composables'),
            '@stores': resolve(__dirname, 'src/stores'),
            '@utils': resolve(__dirname, 'src/utils'),
            '@types': resolve(__dirname, 'src/types'),
            '@assets': resolve(__dirname, 'src/assets'),
            '@repo-assets': resolve(__dirname, '../assets'),
            '@hooks': resolve(__dirname, 'src/hooks'),
            '@layouts': resolve(__dirname, 'src/layouts'),
            '@shell': resolve(__dirname, 'src/shell'),
            '@providers': resolve(__dirname, 'src/providers'),
        },
    },
    server: {
        port: 1420,
        strictPort: true,
    },
    build: {
        target: 'esnext',
        minify: true,
        rollupOptions: {
            output: {
                // 将稳定依赖拆出，降低入口 chunk 大小并提高升级后的浏览器缓存命中率。
                manualChunks(id) {
                    if (!id.includes('node_modules')) return undefined
                    if (/node_modules\/(vue|vue-router|vue-i18n|pinia|@vueuse)\//.test(id)) {
                        return 'framework'
                    }
                    if (/node_modules\/(naive-ui|lucide-vue-next)\//.test(id)) {
                        return 'ui'
                    }
                    return 'vendor'
                },
            },
        },
    },
})
