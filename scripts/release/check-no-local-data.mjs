#!/usr/bin/env node
import { execFile } from 'node:child_process'
import { stat } from 'node:fs/promises'
import path from 'node:path'
import { promisify } from 'node:util'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(scriptDir, '../..')

const forbiddenExactNames = new Set([
  'client-runtime.json',
  'client-config.json',
  'projects.sqlite3',
  'domains.sqlite3',
])

const forbiddenExtensions = new Set(['.sqlite3', '.sqlite', '.db', '.gatebackup'])

const forbiddenCertificateNames = new Set(['private_key.pem', 'certificate.pem'])

const violations = []
const execFileAsync = promisify(execFile)

// 只检查 Git 会纳入版本控制的文件；已被 .gitignore 排除的本机运行数据不会进入发布源码。
async function releaseInputFiles() {
  const { stdout } = await execFileAsync(
    'git',
    ['ls-files', '--cached', '--others', '--exclude-standard', '-z'],
    { cwd: root, encoding: 'buffer', maxBuffer: 16 * 1024 * 1024 },
  )
  return stdout.toString('utf8').split('\0').filter(Boolean)
}

for (const relativePath of await releaseInputFiles()) {
  const fullPath = path.join(root, relativePath)
  const lowerName = path.basename(relativePath).toLowerCase()
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

if (violations.length > 0) {
  console.error('Release data guard failed: 本地运行数据/证书文件不能进入源码或构建输入。')
  for (const item of violations) console.error(` - ${item}`)
  console.error(
    '请移除这些文件，或放到平台应用数据目录（开发版默认 Gate-dev，正式版默认 Gate-release）。',
  )
  process.exit(1)
}

console.log('Release data guard passed: 未发现会被误打包的本地运行数据。')
