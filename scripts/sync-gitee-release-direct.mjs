// 不依赖 GitHub API，按当前版本的完整资产清单将 GitHub Release 镜像到 Gitee。
import { createWriteStream, existsSync, mkdirSync, readFileSync, rmSync } from 'node:fs'
import { join } from 'node:path'
import { pipeline } from 'node:stream/promises'

const tag = process.argv[2]
const token = process.env.GITEE_TOKEN
const version = tag?.replace(/^v/, '')
const ghBase = `https://github.com/Somirk134/Gate/releases/download/${tag}`

const ASSETS = [
  `Gate_${version}_x64-setup.exe`,
  `Gate_${version}_x64-setup.exe.sig`,
  `Gate_${version}_x64.dmg`,
  `Gate_${version}_aarch64.dmg`,
  `Gate_${version}_macos-x64.app.tar.gz`,
  `Gate_${version}_macos-x64.app.tar.gz.sig`,
  `Gate_${version}_macos-arm64.app.tar.gz`,
  `Gate_${version}_macos-arm64.app.tar.gz.sig`,
  `Gate_${version}_amd64.AppImage`,
  `Gate_${version}_amd64.AppImage.sig`,
  `Gate_${version}_amd64.deb`,
  `gate-server-${tag}-linux-x64`,
  `gate-server-${tag}-macos-arm64`,
  `gate-server-${tag}-macos-x64`,
  `gate-server-${tag}-windows-x64.exe`,
  'latest.json',
]

if (!tag || !/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag) || !token) {
  console.error('用法: GITEE_TOKEN=... node scripts/sync-gitee-release-direct.mjs v0.9.2')
  process.exit(1)
}

async function gitee(method, path, { json, form } = {}) {
  const url = new URL(`https://gitee.com/api/v5${path}`)
  url.searchParams.set('access_token', token)
  const init = { method }
  if (json) {
    init.headers = { 'Content-Type': 'application/json' }
    init.body = JSON.stringify(json)
  } else if (form) init.body = form
  const res = await fetch(url, init)
  const text = await res.text()
  if (!res.ok) throw new Error(`Gitee ${method} ${path} -> ${res.status} ${text}`)
  return text ? JSON.parse(text) : null
}

async function download(url, dest) {
  const res = await fetch(url, { redirect: 'follow' })
  if (!res.ok) throw new Error(`下载失败 ${url} -> ${res.status}`)
  await pipeline(res.body, createWriteStream(dest))
}

async function main() {
  let release = null
  try {
    release = await gitee(
      'GET',
      `/repos/lancemorii-git/gate/releases/tags/${encodeURIComponent(tag)}`,
    )
  } catch {
    release = null
  }

  if (!release?.id) {
    release = await gitee('POST', '/repos/lancemorii-git/gate/releases', {
      json: {
        tag_name: tag,
        name: `Gate ${tag}`,
        body: `Gate ${tag} release (mirrored from GitHub).`,
        target_commitish: 'master',
      },
    })
    console.log(`已创建 Gitee Release #${release.id}`)
  } else {
    console.log(`Gitee Release 已存在 #${release.id}`)
  }

  const current = await gitee('GET', `/repos/lancemorii-git/gate/releases/${release.id}`)
  const existing = new Set((current.assets ?? []).map((a) => a.name))
  const tmp = join(process.cwd(), '.gitee-release-sync')
  mkdirSync(tmp, { recursive: true })

  for (const name of ASSETS) {
    if (existing.has(name)) {
      console.log(`跳过: ${name}`)
      continue
    }
    const local = join(tmp, name)
    console.log(`下载 ${name} ...`)
    await download(`${ghBase}/${name}`, local)
    const form = new FormData()
    form.append('file', new Blob([readFileSync(local)]), name)
    await gitee('POST', `/repos/lancemorii-git/gate/releases/${release.id}/attach_files`, { form })
    console.log(`已上传: ${name}`)
  }

  rmSync(tmp, { recursive: true, force: true })
  console.log(`完成: https://gitee.com/lancemorii-git/gate/releases/tag/${tag}`)
}

main().catch((err) => {
  console.error(err)
  process.exit(1)
})
