#!/usr/bin/env node
'use strict';

const fs = require('fs');
const https = require('https');
const path = require('path');

const pkg = require('../package.json');
const version = `v${pkg.version}`;
const repo = 'centy-io/centy-daemon';

const TARGETS = {
  'darwin-arm64': 'centy-mcp-darwin-aarch64',
  'darwin-x64':   'centy-mcp-darwin-x86_64',
  'linux-arm64':  'centy-mcp-linux-aarch64',
  'linux-x64':    'centy-mcp-linux-x86_64',
  'win32-x64':    'centy-mcp-windows-x86_64.exe',
};

const key = `${process.platform}-${process.arch}`;
const assetName = TARGETS[key];

if (!assetName) {
  console.error(`[centy-mcp] Unsupported platform: ${key}`);
  process.exit(1);
}

const isWin = process.platform === 'win32';
const binName = isWin ? 'centy-mcp.exe' : 'centy-mcp';
const binPath = path.join(__dirname, '..', 'bin', binName);

if (fs.existsSync(binPath)) {
  process.exit(0);
}

const url = `https://github.com/${repo}/releases/download/${version}/${assetName}`;
console.log(`[centy-mcp] Downloading ${assetName}...`);

function download(url, dest, redirects) {
  if (redirects === 0) throw new Error('Too many redirects');
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      if (res.statusCode === 301 || res.statusCode === 302) {
        return download(res.headers.location, dest, redirects - 1).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode} downloading ${url}`));
      }
      const out = fs.createWriteStream(dest);
      res.pipe(out);
      out.on('finish', resolve);
      out.on('error', reject);
    }).on('error', reject);
  });
}

download(url, binPath, 5)
  .then(() => {
    if (!isWin) fs.chmodSync(binPath, 0o755);
    console.log('[centy-mcp] Ready.');
  })
  .catch((err) => {
    console.error(`[centy-mcp] Download failed: ${err.message}`);
    console.error(`[centy-mcp] Download manually from:\n  ${url}`);
    process.exit(1);
  });
