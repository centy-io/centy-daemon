#!/usr/bin/env node
'use strict';

const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const isWin = process.platform === 'win32';
const binName = isWin ? 'centy-mcp.exe' : 'centy-mcp';
const binPath = path.join(__dirname, binName);

if (!fs.existsSync(binPath)) {
  console.error('[centy-mcp] Binary not found. Try: npx centy-mcp@latest');
  process.exit(1);
}

const result = spawnSync(binPath, process.argv.slice(2), { stdio: 'inherit' });
process.exit(result.status ?? 1);
