// 合并各平台构建产出的 latest-*.json 为统一 latest.json（含全部平台条目），
// 供 Tauri updater 的 GitHub endpoint 使用。
// 用法: node scripts/merge-updater-manifests.mjs <输入目录> [输出路径]
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { dirname, join } from 'node:path'

const inputDir = process.argv[2]
const outPath = process.argv[3] || 'dist/latest.json'

if (!inputDir) {
  console.error('用法: node scripts/merge-updater-manifests.mjs <输入目录> [输出路径]')
  process.exit(1)
}

function collectManifestFiles(dir, acc = []) {
  if (!existsSync(dir)) return acc
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name)
    if (entry.isDirectory()) collectManifestFiles(full, acc)
    else if (entry.name.startsWith('latest-') && entry.name.endsWith('.json')) acc.push(full)
  }
  return acc
}

const files = collectManifestFiles(inputDir)

if (files.length === 0) {
  console.error(`未在 ${inputDir} 找到 latest-*.json`)
  process.exit(1)
}

const merged = { platforms: {} }
let meta = null

for (const file of files) {
  const data = JSON.parse(readFileSync(file, 'utf8'))
  if (!meta) {
    meta = {
      version: data.version,
      notes: data.notes,
      pub_date: data.pub_date,
    }
  }
  // 首次写入优先，避免 deb 的 linux 条目覆盖 appimage 的同类条目。
  for (const [key, value] of Object.entries(data.platforms || {})) {
    if (!merged.platforms[key]) merged.platforms[key] = value
  }
}

if (meta) Object.assign(merged, meta)

mkdirSync(dirname(outPath), { recursive: true })
writeFileSync(outPath, JSON.stringify(merged, null, 2))
console.log(`已合并 ${files.length} 个平台清单 -> ${outPath}`)
