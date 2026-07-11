// 从 Tauri bundle 目录中的签名更新包生成单平台 latest.json 片段。
// Tauri 只生成 .sig，不生成 latest.json；此脚本在 CI 各平台构建后调用。
// 用法: node scripts/generate-platform-updater-manifest.mjs <bundle根目录...> --tag vX.Y.Z --platform windows-x64 --out updater/latest-windows-x64-0.json
import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync } from 'node:fs'
import { basename, dirname, join, relative } from 'node:path'

const PLATFORM_KEYS = {
  'windows-x64': 'windows-x86_64',
  'macos-x64': 'darwin-x86_64',
  'macos-arm64': 'darwin-aarch64',
  'linux-x64': 'linux-x86_64',
}

function parseArgs(argv) {
  const bundleRoots = []
  let tag = ''
  let platform = ''
  let out = ''

  for (let i = 2; i < argv.length; i++) {
    const arg = argv[i]
    if (arg === '--tag') tag = argv[++i] ?? ''
    else if (arg === '--platform') platform = argv[++i] ?? ''
    else if (arg === '--out') out = argv[++i] ?? ''
    else if (!arg.startsWith('--')) bundleRoots.push(arg)
  }

  const releaseRoots = bundleRoots.map((root) =>
    root.replace(/\/bundle\/?$/, '/release').replace(/\\bundle\\?$/, '\\release'),
  )

  return { bundleRoots, releaseRoots, tag, platform, out }
}

function walkFiles(dir, files = []) {
  if (!dir) return files
  let entries
  try {
    entries = readdirSync(dir)
  } catch {
    return files
  }
  for (const entry of entries) {
    const full = join(dir, entry)
    let st
    try {
      st = statSync(full)
    } catch {
      continue
    }
    if (st.isDirectory()) walkFiles(full, files)
    else files.push(full)
  }
  return files
}

function stripVersion(tag) {
  return tag.replace(/^v/i, '')
}

function artifactForSig(sigPath, files) {
  const withoutSig = sigPath.endsWith('.sig') ? sigPath.slice(0, -4) : sigPath
  if (files.includes(withoutSig)) return withoutSig

  const base = basename(withoutSig)
  const dir = dirname(withoutSig)
  const candidates = files.filter((f) => {
    if (!f.startsWith(dir)) return false
    const name = basename(f)
    return name === base || name.startsWith(`${base}.`) || `${name}.sig` === basename(sigPath)
  })
  return candidates.sort((a, b) => basename(a).length - basename(b).length)[0]
}

function pickUpdaterArtifact(files, platform) {
  const sigFiles = files.filter((f) => f.endsWith('.sig'))
  const pairs = []

  for (const sig of sigFiles) {
    const artifact = artifactForSig(sig, files)
    if (!artifact) continue
    const name = basename(artifact).toLowerCase()
    const rel = artifact.replace(/\\/g, '/').toLowerCase()
    if (platform.startsWith('windows') && !name.endsWith('.exe') && !name.endsWith('.msi')) continue
    if (platform.startsWith('macos')) {
      const isMacUpdater =
        name.endsWith('.app.tar.gz') ||
        name.endsWith('.tar.gz') && (rel.includes('/macos/') || rel.includes('/osx/'))
      if (!isMacUpdater) continue
    }
    if (platform.startsWith('linux')) {
      const isLinuxUpdater =
        name.endsWith('.appimage') ||
        name.endsWith('.appimage.tar.gz') ||
        (name.endsWith('.tar.gz') && rel.includes('/appimage/'))
      if (!isLinuxUpdater) continue
    }
    pairs.push({ artifact, sig })
  }

  if (pairs.length === 0) return null

  const priority = (artifact) => {
    const name = basename(artifact).toLowerCase()
    if (name.endsWith('.app.tar.gz')) return 0
    if (name.endsWith('.appimage')) return 1
    if (name.endsWith('.appimage.tar.gz')) return 2
    if (name.endsWith('-setup.exe')) return 0
    if (name.endsWith('.msi')) return 1
    if (name.endsWith('.exe')) return 2
    if (name.endsWith('.appimage')) return 3
    return 10
  }

  pairs.sort((a, b) => priority(a.artifact) - priority(b.artifact))
  return pairs[0]
}

function describeBundleFiles(files) {
  const interesting = files.filter((f) => /\.(sig|tar\.gz|exe|msi|appimage|dmg)$/i.test(f))
  return interesting.map((f) => f.replace(/\\/g, '/')).sort()
}

const { bundleRoots, releaseRoots, tag, platform, out } = parseArgs(process.argv)
const platformKey = PLATFORM_KEYS[platform]

if (!tag || !platform || !out || !platformKey || bundleRoots.length === 0) {
  console.error(
    '用法: node scripts/generate-platform-updater-manifest.mjs <bundle根目录...> --tag vX.Y.Z --platform <matrix.name> --out <输出路径>',
  )
  process.exit(1)
}

const searchRoots = [...new Set([...bundleRoots, ...releaseRoots])]
const allFiles = searchRoots.flatMap((root) => walkFiles(root))
const picked = pickUpdaterArtifact(allFiles, platform)

if (!picked) {
  console.error(`未在 ${searchRoots.join(', ')} 找到 ${platform} 的签名更新包`)
  const sample = describeBundleFiles(allFiles)
  if (sample.length > 0) {
    console.error('可见产物:')
    for (const line of sample.slice(0, 40)) console.error(`  - ${line}`)
    if (sample.length > 40) console.error(`  ... 另有 ${sample.length - 40} 个文件`)
  }
  process.exit(1)
}

const version = stripVersion(tag)
const fileName = basename(picked.artifact)
const signature = readFileSync(picked.sig, 'utf8').trim()
const downloadBase = `https://github.com/Somirk134/Gate/releases/download/${tag}`

const manifest = {
  version,
  notes: `Gate ${tag}`,
  pub_date: new Date().toISOString(),
  platforms: {
    [platformKey]: {
      signature,
      url: `${downloadBase}/${fileName}`,
    },
  },
}

mkdirSync(dirname(out), { recursive: true })
writeFileSync(out, JSON.stringify(manifest, null, 2))
console.log(`已生成 ${platform} updater 清单 -> ${out} (${relative(process.cwd(), picked.artifact)})`)
