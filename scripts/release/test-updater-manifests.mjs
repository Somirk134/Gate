#!/usr/bin/env node
import assert from 'node:assert/strict'
import { execFileSync, spawnSync } from 'node:child_process'
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const root = path.resolve(scriptDir, '../..')
const generateScript = path.join(root, 'scripts/generate-platform-updater-manifest.mjs')
const mergeScript = path.join(root, 'scripts/merge-updater-manifests.mjs')
const tempRoot = mkdtempSync(path.join(tmpdir(), 'gate-updater-test-'))
const updaterDir = path.join(tempRoot, 'updater')
const distDir = path.join(tempRoot, 'dist')
const testVersion = '9.8.7'
const testTag = `v${testVersion}`

function fixture(platform, files) {
  const bundleRoot = path.join(tempRoot, platform, 'release', 'bundle')
  for (const [relativePath, content] of Object.entries(files)) {
    const fullPath = path.join(bundleRoot, relativePath)
    mkdirSync(path.dirname(fullPath), { recursive: true })
    writeFileSync(fullPath, content)
  }
  return bundleRoot
}

function generate(platform, files) {
  const bundleRoot = fixture(platform, files)
  execFileSync(
    process.execPath,
    [
      generateScript,
      bundleRoot,
      '--tag',
      testTag,
      '--platform',
      platform,
      '--out',
      path.join(updaterDir, `latest-${platform}.json`),
      '--assets-out',
      distDir,
    ],
    { stdio: 'pipe' },
  )
}

try {
  generate('windows-x64', {
    [`nsis/Gate_${testVersion}_x64-setup.exe`]: 'windows updater',
    [`nsis/Gate_${testVersion}_x64-setup.exe.sig`]: 'windows-signature',
  })
  generate('macos-x64', {
    [`dmg/Gate_${testVersion}_x64.dmg`]: 'macos x64 installer',
    'macos/Gate.app.tar.gz': 'macos x64 updater',
    'macos/Gate.app.tar.gz.sig': 'macos-x64-signature',
  })
  generate('macos-arm64', {
    [`dmg/Gate_${testVersion}_aarch64.dmg`]: 'macos arm64 installer',
    'macos/Gate.app.tar.gz': 'macos arm64 updater',
    'macos/Gate.app.tar.gz.sig': 'macos-arm64-signature',
  })
  generate('linux-x64', {
    [`appimage/Gate_${testVersion}_amd64.AppImage`]: 'linux updater',
    [`appimage/Gate_${testVersion}_amd64.AppImage.sig`]: 'linux-signature',
    [`deb/Gate_${testVersion}_amd64.deb`]: 'linux installer',
  })

  execFileSync(process.execPath, [mergeScript, updaterDir, path.join(distDir, 'latest.json')], {
    stdio: 'pipe',
  })

  const latest = JSON.parse(readFileSync(path.join(distDir, 'latest.json'), 'utf8'))
  assert.equal(latest.version, testVersion)
  assert.deepEqual(Object.keys(latest.platforms).sort(), [
    'darwin-aarch64',
    'darwin-x86_64',
    'linux-x86_64',
    'windows-x86_64',
  ])

  const urls = Object.values(latest.platforms).map((platform) => platform.url)
  assert.equal(new Set(urls).size, 4, '四个平台必须使用不同的 updater 资产')
  for (const platform of Object.values(latest.platforms)) {
    assert.ok(platform.signature, 'updater 签名不能为空')
    assert.ok(existsSync(path.join(distDir, path.basename(new URL(platform.url).pathname))))
  }
  assert.ok(existsSync(path.join(distDir, `Gate_${testVersion}_macos-x64.app.tar.gz.sig`)))
  assert.ok(existsSync(path.join(distDir, `Gate_${testVersion}_macos-arm64.app.tar.gz.sig`)))

  // 缺少任一平台时合并必须失败，避免发布不完整 latest.json。
  const missingDir = path.join(tempRoot, 'missing-platform')
  mkdirSync(missingDir, { recursive: true })
  writeFileSync(
    path.join(missingDir, 'latest-windows-x64.json'),
    readFileSync(path.join(updaterDir, 'latest-windows-x64.json')),
  )
  const missingResult = spawnSync(
    process.execPath,
    [mergeScript, missingDir, path.join(tempRoot, 'invalid-dist', 'latest.json')],
    { encoding: 'utf8' },
  )
  assert.notEqual(missingResult.status, 0, '缺少平台的 updater 清单不应通过')

  console.log('Release updater manifest tests passed.')
} finally {
  rmSync(tempRoot, { recursive: true, force: true })
}
