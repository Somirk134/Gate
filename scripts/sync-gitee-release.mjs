// 将 GitHub Release 资产镜像到 Gitee Release（资产完全一致）。
// 需要环境变量 GITEE_TOKEN（私人令牌，repo 权限）。
// 用法: node scripts/sync-gitee-release.mjs v0.9.2
import { createWriteStream, existsSync, mkdirSync, readFileSync, rmSync } from 'node:fs'
import { basename, join } from 'node:path'
import { pipeline } from 'node:stream/promises'

const GITHUB_OWNER = 'Somirk134'
const GITHUB_REPO = 'Gate'
const GITEE_OWNER = 'lancemorii-git'
const GITEE_REPO = 'gate'

const tag = process.argv[2]
const token = process.env.GITEE_TOKEN

if (!tag || !token) {
  console.error('用法: GITEE_TOKEN=... node scripts/sync-gitee-release.mjs v0.9.2')
  process.exit(1)
}

async function gh(path) {
  const res = await fetch(`https://api.github.com${path}`, {
    headers: { Accept: 'application/vnd.github+json', 'User-Agent': 'gate-release-sync' },
  })
  if (!res.ok) throw new Error(`GitHub ${path} -> ${res.status} ${await res.text()}`)
  return res.json()
}

async function gitee(method, path, { json, form } = {}) {
  const url = new URL(`https://gitee.com/api/v5${path}`)
  url.searchParams.set('access_token', token)
  const init = { method }
  if (json) {
    init.headers = { 'Content-Type': 'application/json' }
    init.body = JSON.stringify(json)
  } else if (form) {
    init.body = form
  }
  const res = await fetch(url, init)
  const text = await res.text()
  let body
  try {
    body = text ? JSON.parse(text) : null
  } catch {
    body = text
  }
  if (!res.ok) throw new Error(`Gitee ${method} ${path} -> ${res.status} ${text}`)
  return body
}

async function download(url, dest) {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`下载失败 ${url} -> ${res.status}`)
  await pipeline(res.body, createWriteStream(dest))
}

async function main() {
  const ghRelease = await gh(`/repos/${GITHUB_OWNER}/${GITHUB_REPO}/releases/tags/${tag}`)
  const assets = ghRelease.assets ?? []
  if (assets.length === 0) throw new Error(`GitHub Release ${tag} 尚无资产`)

  const existing = await gitee(
    'GET',
    `/repos/${GITEE_OWNER}/${GITEE_REPO}/releases/tags/${encodeURIComponent(tag)}`,
  ).catch(() => null)

  let release = existing
  if (!release || !release.id) {
    release = await gitee('POST', `/repos/${GITEE_OWNER}/${GITEE_REPO}/releases`, {
      json: {
        tag_name: tag,
        name: ghRelease.name || `Gate ${tag}`,
        body: ghRelease.body || `Gate ${tag} release (mirrored from GitHub).`,
        target_commitish: 'master',
        prerelease: !!ghRelease.prerelease,
      },
    })
    console.log(`已创建 Gitee Release #${release.id}`)
  } else {
    console.log(`Gitee Release 已存在 #${release.id}，将补齐资产`)
  }

  const current = await gitee('GET', `/repos/${GITEE_OWNER}/${GITEE_REPO}/releases/${release.id}`)
  const existingNames = new Set((current.assets ?? []).map((a) => a.name))
  const tmpDir = join(process.cwd(), '.gitee-release-sync')
  mkdirSync(tmpDir, { recursive: true })

  for (const asset of assets) {
    if (existingNames.has(asset.name)) {
      console.log(`跳过已存在: ${asset.name}`)
      continue
    }
    const localPath = join(tmpDir, asset.name)
    console.log(`下载 ${asset.name} ...`)
    await download(asset.browser_download_url, localPath)

    const form = new FormData()
    form.append('file', new Blob([readFileSync(localPath)]), asset.name)
    await gitee('POST', `/repos/${GITEE_OWNER}/${GITEE_REPO}/releases/${release.id}/attach_files`, {
      form,
    })
    console.log(`已上传: ${asset.name}`)
  }

  if (existsSync(tmpDir)) rmSync(tmpDir, { recursive: true, force: true })
  console.log(
    `Gitee Release 同步完成: https://gitee.com/${GITEE_OWNER}/${GITEE_REPO}/releases/tag/${tag}`,
  )
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
