import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers'
import { resolve } from 'path'

export default defineConfig({
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
            '@stores': resolve(__dirname, 'src/stores'),
            '@api': resolve(__dirname, 'src/api'),
            '@utils': resolve(__dirname, 'src/utils'),
            '@types': resolve(__dirname, 'src/types'),
            '@assets': resolve(__dirname, 'src/assets'),
            '@hooks': resolve(__dirname, 'src/hooks'),
        },
    },
    server: {
        port: 1420,
        strictPort: true,
    },
    build: {
        target: 'esnext',
        minify: 'esbuild',
    },
})
