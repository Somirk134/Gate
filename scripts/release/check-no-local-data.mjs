#!/usr/bin/env node
import { readdir, stat } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(scriptDir, '../..')

const ignoredDirectories = new Set([
  '.git',
  '.idea',
  '.vscode',
  'node_modules',
  'target',
  '.cache',
])

const forbiddenExactNames = new Set([
  'client-runtime.json',
  'client-config.json',
  'projects.sqlite3',
  'domains.sqlite3',
])

const forbiddenExtensions = new Set([
  '.sqlite3',
  '.sqlite',
  '.db',
  '.gatebackup',
])

const forbiddenCertificateNames = new Set([
  'private_key.pem',
  'certificate.pem',
])

const violations = []

async function walk(dir) {
  const entries = await readdir(dir, { withFileTypes: true })
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name)
    const relativePath = path.relative(root, fullPath).replaceAll(path.sep, '/')

    if (entry.isDirectory()) {
      if (ignoredDirectories.has(entry.name)) continue
      await walk(fullPath)
      continue
    }

    if (!entry.isFile()) continue

    const lowerName = entry.name.toLowerCase()
    const ext = path.extname(lowerName)
    const fileStat = await stat(fullPath)

    if (
      forbiddenExactNames.has(lowerName) ||
      forbiddenExtensions.has(ext) ||
      forbiddenCertificateNames.has(lowerName)
    ) {
      violations.push(`${relativePath} (${fileStat.size} bytes)`)
    }
  }
}

await walk(root)

if (violations.length > 0) {
  console.error('Release data guard failed: 本地运行数据/证书文件不能进入源码或构建输入。')
  for (const item of violations) console.error(` - ${item}`)
  console.error('请移除这些文件，或放到平台应用数据目录（开发版默认 Gate-dev，正式版默认 Gate-release）。')
  process.exit(1)
}

console.log('Release data guard passed: 未发现会被误打包的本地运行数据。')
