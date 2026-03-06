#!/usr/bin/env node
/* eslint-disable no-console */

import fs from 'node:fs/promises'
import path from 'node:path'
import { gzipSync } from 'node:zlib'

const assetsDir = path.resolve(process.cwd(), 'dist/assets')

const kib = (bytes) => (bytes / 1024).toFixed(2)

const ensureAssetsDir = async () => {
  try {
    const stat = await fs.stat(assetsDir)
    if (!stat.isDirectory()) {
      throw new Error('not a directory')
    }
  } catch {
    console.error(`[bundle-budget] Missing dist assets directory: ${assetsDir}`)
    process.exit(1)
  }
}

const findLargestChunk = async (files, prefix) => {
  const candidates = files.filter((file) => file.startsWith(prefix) && file.endsWith('.js'))
  let largest = null
  let largestSize = -1

  for (const file of candidates) {
    const stat = await fs.stat(path.join(assetsDir, file))
    if (stat.size > largestSize) {
      largest = file
      largestSize = stat.size
    }
  }

  return largest
}

const getFileMetrics = async (fileName) => {
  const absPath = path.join(assetsDir, fileName)
  const content = await fs.readFile(absPath)
  return {
    fileName,
    size: content.byteLength,
    gzipSize: gzipSync(content, { level: 9 }).byteLength,
  }
}

const validateBudget = (name, metrics, budget) => {
  const errors = []
  if (metrics.size > budget.maxBytes) {
    errors.push(`${name} raw ${kib(metrics.size)} KiB > ${kib(budget.maxBytes)} KiB`)
  }
  if (typeof budget.maxGzipBytes === 'number' && metrics.gzipSize > budget.maxGzipBytes) {
    errors.push(`${name} gzip ${kib(metrics.gzipSize)} KiB > ${kib(budget.maxGzipBytes)} KiB`)
  }
  return errors
}

await ensureAssetsDir()
const files = await fs.readdir(assetsDir)

const usageChunk = await findLargestChunk(files, 'UsageDashboardView-')
if (!usageChunk) {
  console.error('[bundle-budget] Missing UsageDashboardView chunk in dist/assets')
  process.exit(1)
}

const entryChunk = await findLargestChunk(files, 'index-')
if (!entryChunk) {
  console.error('[bundle-budget] Missing index chunk in dist/assets')
  process.exit(1)
}

const usageMetrics = await getFileMetrics(usageChunk)
const entryMetrics = await getFileMetrics(entryChunk)

const failures = [
  ...validateBudget('UsageDashboardView', usageMetrics, {
    maxBytes: 250 * 1024,
    maxGzipBytes: 80 * 1024,
  }),
  ...validateBudget('index', entryMetrics, {
    maxBytes: 150 * 1024,
  }),
]

console.log(`[bundle-budget] UsageDashboardView: ${usageMetrics.fileName} raw=${kib(usageMetrics.size)} KiB gzip=${kib(usageMetrics.gzipSize)} KiB`)
console.log(`[bundle-budget] index: ${entryMetrics.fileName} raw=${kib(entryMetrics.size)} KiB gzip=${kib(entryMetrics.gzipSize)} KiB`)

if (failures.length > 0) {
  for (const failure of failures) {
    console.error(`[bundle-budget] FAIL ${failure}`)
  }
  process.exit(1)
}

console.log('[bundle-budget] PASS all budgets satisfied')
