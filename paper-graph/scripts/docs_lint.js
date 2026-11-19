#!/usr/bin/env node
'use strict';

const fs = require('node:fs');
const path = require('node:path');

const repoRoot = path.resolve(process.argv[2]);
const crateRoot = path.resolve(process.argv[3]);
const knowledgeRoot = path.join(crateRoot, 'knowledge-system');
const projectDocsRoot = path.join(crateRoot, 'docs');
const ownerMapPath = path.join(knowledgeRoot, 'docs', 'owner-map.md');
const gatesPath = path.join(crateRoot, 'experiments', 'ontology-extraction', 'gates.md');
const verificationPath = path.join(projectDocsRoot, 'VERIFICATION.md');

const normativeFiles = [
  path.join(knowledgeRoot, 'CONSTITUTION.md'),
  path.join(knowledgeRoot, 'method.md'),
  ...fs.readdirSync(projectDocsRoot)
    .filter(name => name.endsWith('.md'))
    .sort()
    .map(name => path.join(projectDocsRoot, name)),
];
const lintFiles = [...normativeFiles, ownerMapPath, gatesPath];
const errors = [];

function relative(file) {
  return path.relative(repoRoot, file).split(path.sep).join('/');
}

function readLines(file) {
  return fs.readFileSync(file, 'utf8').split(/\r?\n/u);
}

function outsideFences(lines) {
  const result = [];
  let fence = null;
  for (let index = 0; index < lines.length; index += 1) {
    const match = lines[index].match(/^\s*(`{3,}|~{3,})/u);
    if (match) {
      if (fence === null) fence = match[1][0];
      else if (match[1][0] === fence) fence = null;
      continue;
    }
    if (fence === null) result.push({ number: index + 1, text: lines[index] });
  }
  return result;
}

function fail(file, line, check, message) {
  errors.push(`${relative(file)}:${line}: ${check}: ${message}`);
}

// (a) Resolve every relative Markdown link whose target ends in .md.
for (const file of lintFiles) {
  for (const { number, text } of outsideFences(readLines(file))) {
    const destinations = [];
    for (const match of text.matchAll(/!?\[[^\]]*\]\(([^)]+)\)/gu)) {
      destinations.push(match[1]);
    }
    const reference = text.match(/^\s*\[[^\]]+\]:\s+(\S+)/u);
    if (reference) destinations.push(reference[1]);

    for (let destination of destinations) {
      destination = destination.trim();
      if (destination.startsWith('<')) {
        const close = destination.indexOf('>');
        if (close === -1) continue;
        destination = destination.slice(1, close);
      } else {
        destination = destination.split(/\s+/u)[0];
      }
      if (/^(?:[a-z][a-z0-9+.-]*:|\/|#|~)/iu.test(destination)) continue;
      const targetPath = destination.split('#')[0].split('?')[0];
      if (!/\.md$/iu.test(targetPath)) continue;
      let decoded;
      try {
        decoded = decodeURIComponent(targetPath);
      } catch {
        fail(file, number, 'link', `invalid percent encoding in ${destination}`);
        continue;
      }
      const resolved = path.resolve(path.dirname(file), decoded);
      if (!fs.existsSync(resolved) || !fs.statSync(resolved).isFile()) {
        fail(file, number, 'link', `unresolved target ${destination}`);
      }
    }
  }
}

// (b) Validate exactly one owner and one resolvable check per subject row.
const checkIds = new Set(
  fs.readdirSync(path.join(crateRoot, 'tests'))
    .filter(name => /^check_.*\.rs$/u.test(name))
    .map(name => name.slice(0, -3)),
);
const ownerLines = readLines(ownerMapPath);
const subjectHeading = ownerLines.findIndex(line => line.trim() === '## Subject Ownership');
if (subjectHeading === -1) {
  fail(ownerMapPath, 1, 'owner-map', 'missing ## Subject Ownership section');
} else {
  let rowCount = 0;
  for (let index = subjectHeading + 1; index < ownerLines.length; index += 1) {
    const text = ownerLines[index];
    if (/^##\s/u.test(text) || /^---\s*$/u.test(text)) break;
    if (!text.startsWith('|')) continue;
    const cells = text.split('|').slice(1, -1).map(cell => cell.trim());
    if (cells.length !== 5 || cells[0] === 'Subject' || /^:?-+:?$/u.test(cells[0])) continue;
    rowCount += 1;
    const line = index + 1;
    const ownerTokens = [...cells[2].matchAll(/`([^`]+)`/gu)].map(match => match[1]);
    const ownerRemainder = cells[2].replace(/`[^`]+`/gu, '').trim();
    if (ownerTokens.length !== 1 || ownerRemainder !== '') {
      fail(ownerMapPath, line, 'owner-map', `expected exactly one owner, found ${cells[2] || '(empty)'}`);
    }
    const checkTokens = [...cells[3].matchAll(/`([^`]+)`/gu)].map(match => match[1]);
    const checkRemainder = cells[3].replace(/`[^`]+`/gu, '').trim();
    if (checkTokens.length !== 1 || checkRemainder !== '') {
      fail(ownerMapPath, line, 'owner-map', `expected exactly one check id or judgment (human), found ${cells[3] || '(empty)'}`);
    } else if (checkTokens[0] !== 'judgment (human)' &&
               (!/^check_/u.test(checkTokens[0]) || !checkIds.has(checkTokens[0]))) {
      fail(ownerMapPath, line, 'owner-map', `unknown check id ${checkTokens[0]}`);
    }
  }
  if (rowCount === 0) fail(ownerMapPath, subjectHeading + 1, 'owner-map', 'no subject rows found');
}

// (c) A completed gate may not cite a command absent from this tree.
function pythonTestsExist(directory) {
  if (!fs.existsSync(directory)) return false;
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    if (entry.name.startsWith('.') || entry.name === 'target' || entry.name === 'node_modules') continue;
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      if (pythonTestsExist(full)) return true;
    } else if (/^(?:test_.*|.*_test)\.py$/u.test(entry.name)) {
      return true;
    }
  }
  return false;
}
const hasPythonTests = pythonTestsExist(crateRoot);

function validateCommand(file, line, command) {
  const normalized = command.replace(/\s+#.*$/u, '').trim();
  const bashMatch = normalized.match(/^bash\s+([^\s]+)/u);
  if (bashMatch) {
    const script = bashMatch[1];
    const resolved = script.startsWith('paper-graph/')
      ? path.join(repoRoot, script)
      : path.resolve(crateRoot, script);
    if (!fs.existsSync(resolved) || !fs.statSync(resolved).isFile()) {
      fail(file, line, 'stale-command', `missing script for ${normalized}`);
    }
    return;
  }
  if (/^cargo\s+test\b/u.test(normalized)) {
    if (!fs.existsSync(path.join(crateRoot, 'Cargo.toml'))) {
      fail(file, line, 'stale-command', `missing Cargo.toml for ${normalized}`);
      return;
    }
    const targets = [...normalized.matchAll(/(?:^|\s)--test(?:=|\s+)([^\s]+)/gu)].map(match => match[1]);
    for (const target of targets) {
      if (!fs.existsSync(path.join(crateRoot, 'tests', `${target}.rs`))) {
        fail(file, line, 'stale-command', `missing tests/${target}.rs for ${normalized}`);
      }
    }
    return;
  }
  if (/^(?:uv\s+run\s+)?pytest\b/u.test(normalized) && !hasPythonTests) {
    fail(file, line, 'stale-command', `no pytest test files exist for ${normalized}`);
  }
}

function commandsIn(text) {
  const commands = [];
  for (const match of text.matchAll(/`([^`\n]+)`/gu)) {
    if (/^(?:bash|cargo\s+test|uv\s+run\s+pytest|pytest)\b/u.test(match[1].trim())) {
      commands.push(match[1].trim());
    }
  }
  return commands;
}

for (const { number, text } of outsideFences(readLines(gatesPath))) {
  if (!text.startsWith('|')) continue;
  const cells = text.split('|').slice(1, -1).map(cell => cell.trim());
  if (cells.length === 3 && cells[1] === 'yes') {
    for (const command of commandsIn(cells[2])) validateCommand(gatesPath, number, command);
  }
}

const verificationLines = readLines(verificationPath);
for (const { number, text } of outsideFences(verificationLines)) {
  for (const command of commandsIn(text)) validateCommand(verificationPath, number, command);
}
let verificationFence = false;
for (let index = 0; index < verificationLines.length; index += 1) {
  const text = verificationLines[index];
  if (/^\s*```(?:sh|bash|shell)?\s*$/u.test(text)) {
    verificationFence = !verificationFence;
    continue;
  }
  if (verificationFence && /^(?:bash|cargo\s+test|uv\s+run\s+pytest|pytest)\b/u.test(text.trim())) {
    validateCommand(verificationPath, index + 1, text.trim());
  }
}

// (d) Exact duplicate prose lines of at least fifteen words fail across owners.
const occurrences = new Map();
for (const file of normativeFiles) {
  for (const { number, text } of outsideFences(readLines(file))) {
    const normalized = text.trim();
    // Status headers are editorial metadata, not normative sentences.
    if (normalized.startsWith('**Status:**')) continue;
    if (normalized.split(/\s+/u).length < 15) continue;
    if (!occurrences.has(normalized)) occurrences.set(normalized, []);
    occurrences.get(normalized).push({ file, number });
  }
}
for (const [text, places] of occurrences) {
  if (new Set(places.map(place => place.file)).size < 2) continue;
  for (const place of places) {
    const others = places.filter(other => other.file !== place.file)
      .map(other => `${relative(other.file)}:${other.number}`).join(', ');
    fail(place.file, place.number, 'duplicate', `also at ${others}: ${text}`);
  }
}

if (errors.length > 0) {
  console.error(errors.join('\n'));
  console.error(`docs_lint: FAIL (${errors.length} error${errors.length === 1 ? '' : 's'})`);
  process.exit(1);
}
console.log(`docs_lint: PASS (${lintFiles.length} files, ${checkIds.size} check ids)`);
