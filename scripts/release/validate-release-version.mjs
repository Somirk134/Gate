#!/usr/bin/env node
import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(scriptDir, '../..')
const tag = process.argv[2] ?? ''

if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag)) {
  console.error(`发布标签格式无效: ${tag}`)
  process.exit(1)
}

const expected = tag.slice(1).split('-')[0]
const cargo = readFileSync(path.join(root, 'Cargo.toml'), 'utf8')
const workspaceVersion = cargo.match(/\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"/)?.[1]
const rootPackage = JSON.parse(readFileSync(path.join(root, 'package.json'), 'utf8')).version
const clientPackage = JSON.parse(
  readFileSync(path.join(root, 'client/package.json'), 'utf8'),
).version
const tauri = JSON.parse(
  readFileSync(path.join(root, 'client/src-tauri/tauri.conf.json'), 'utf8'),
).version
const tlsCargo = readFileSync(path.join(root, 'server/tls/Cargo.toml'), 'utf8')
const tlsVersion = tlsCargo.match(/\[package\][\s\S]*?version\s*=\s*"([^"]+)"/)?.[1]
const constants = readFileSync(path.join(root, 'client/src/constants/index.ts'), 'utf8')
const appVersion = constants.match(/APP_VERSION\s*=\s*['"]([^'"]+)['"]/)?.[1]

const versions = {
  'Cargo workspace': workspaceVersion,
  'server/tls/Cargo.toml': tlsVersion,
  'root package.json': rootPackage,
  'client package.json': clientPackage,
  'tauri.conf.json': tauri,
  'client APP_VERSION': appVersion,
}
const mismatches = Object.entries(versions).filter(([, version]) => version !== expected)

if (mismatches.length > 0) {
  console.error(`发布标签 ${tag} 与项目版本 ${expected} 不一致:`)
  for (const [name, version] of mismatches) console.error(` - ${name}: ${version ?? 'missing'}`)
  process.exit(1)
}

console.log(`发布版本校验通过: ${tag}`)
