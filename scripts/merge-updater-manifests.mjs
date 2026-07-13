// 合并各平台构建产出的 latest-*.json 为统一 latest.json（含全部平台条目），
// 供 Tauri updater 的 GitHub endpoint 使用。
// 用法: node scripts/merge-updater-manifests.mjs <输入目录> [输出路径]
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs'
import { basename, dirname, join } from 'node:path'

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
const expectedPlatforms = new Set([
  'windows-x86_64',
  'darwin-x86_64',
  'darwin-aarch64',
  'linux-x86_64',
])

for (const file of files) {
  const data = JSON.parse(readFileSync(file, 'utf8'))
  if (!meta) {
    meta = {
      version: data.version,
      notes: data.notes,
      pub_date: data.pub_date,
    }
  } else if (data.version !== meta.version) {
    throw new Error(`平台清单版本不一致: ${file} 为 ${data.version}，预期 ${meta.version}`)
  }
  // 首次写入优先，避免 deb 的 linux 条目覆盖 appimage 的同类条目。
  for (const [key, value] of Object.entries(data.platforms || {})) {
    if (!merged.platforms[key]) merged.platforms[key] = value
  }
}

if (meta) Object.assign(merged, meta)

// 发布前验证四个平台齐全，且清单引用的更新包都已进入 dist。
const actualPlatforms = Object.keys(merged.platforms)
const missingPlatforms = [...expectedPlatforms].filter((platform) => !merged.platforms[platform])
const unexpectedPlatforms = actualPlatforms.filter((platform) => !expectedPlatforms.has(platform))
if (missingPlatforms.length > 0 || unexpectedPlatforms.length > 0) {
  throw new Error(
    `updater 平台集合无效；缺少: ${missingPlatforms.join(', ') || '无'}；多出: ${unexpectedPlatforms.join(', ') || '无'}`,
  )
}

const releaseDir = dirname(outPath)
const referencedUrls = new Set()
for (const [platform, value] of Object.entries(merged.platforms)) {
  const url = value?.url
  if (!url) throw new Error(`${platform} 缺少 updater 下载地址`)
  if (!value.signature?.trim()) throw new Error(`${platform} 缺少 updater 签名`)
  if (referencedUrls.has(url)) throw new Error(`多个平台复用了同一 updater 地址: ${url}`)
  referencedUrls.add(url)

  const assetName = basename(new URL(url).pathname)
  if (!existsSync(join(releaseDir, assetName))) {
    throw new Error(`${platform} updater 资产未上传到 dist: ${assetName}`)
  }
}

mkdirSync(dirname(outPath), { recursive: true })
writeFileSync(outPath, JSON.stringify(merged, null, 2))
console.log(`已合并 ${files.length} 个平台清单 -> ${outPath}`)
