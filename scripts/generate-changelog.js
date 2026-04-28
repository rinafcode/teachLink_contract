#!/usr/bin/env node

/**
 * Automated Changelog Generator
 * 
 * Parses conventional commit messages, categorizes changes, 
 * tracks versions based on git tags, and generates release notes.
 */

const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

function runCommand(command) {
  try {
    return execSync(command, { encoding: 'utf-8' }).trim();
  } catch (error) {
    return '';
  }
}

function generateChangelog() {
  console.log('Generating changelog...');

  // Get tags sorted by semantic version
  const tagsOutput = runCommand('git tag --sort=-v:refname');
  const tags = tagsOutput.split('\n').filter(Boolean);
  
  let currentVersion = 'vUnreleased';
  let previousTag = '';
  let gitLogCmd = 'git log --format="%H|%s"';

  if (tags.length > 0) {
    currentVersion = tags[0];
    if (tags.length > 1) {
      previousTag = tags[1];
      gitLogCmd = `git log ${previousTag}..${currentVersion} --format="%H|%s"`;
    } else {
      gitLogCmd = `git log ${currentVersion} --format="%H|%s"`;
    }
  }

  const logOutput = runCommand(gitLogCmd);
  if (!logOutput) {
    console.log('No commits found for the current range.');
    return;
  }
  
  const commits = logOutput.split('\n').filter(Boolean);

  // Categories for release notes
  const categories = {
    feat: { title: '🚀 Features', items: [] },
    fix: { title: '🐛 Bug Fixes', items: [] },
    security: { title: '🔒 Security', items: [] },
    perf: { title: '⚡ Performance', items: [] },
    refactor: { title: '♻️ Refactoring', items: [] },
    docs: { title: '📚 Documentation', items: [] },
    test: { title: '🧪 Testing', items: [] },
    chore: { title: '🔧 Chores & Maintenance', items: [] },
  };

  const commitRegex = /^(feat|fix|docs|style|refactor|perf|test|chore|security)(?:\(([^)]+)\))?:\s*(.+)$/i;

  commits.forEach(commitLine => {
    const parts = commitLine.split('|');
    if (parts.length < 2) return;
    const hash = parts[0];
    const subject = parts.slice(1).join('|');

    const match = subject.match(commitRegex);
    if (match) {
      const type = match[1].toLowerCase();
      const scope = match[2];
      const message = match[3];

      if (categories[type]) {
        const shortHash = hash.substring(0, 7);
        const scopeStr = scope ? `**${scope}:** ` : '';
        categories[type].items.push(`- ${scopeStr}${message} (${shortHash})`);
      }
    }
  });

  const date = new Date().toISOString().split('T')[0];
  let releaseNotes = `## [${currentVersion}] - ${date}\n\n`;
  
  if (previousTag) {
    releaseNotes += `Compare with ${previousTag}\n\n`;
  }

  let hasChanges = false;
  for (const key in categories) {
    if (categories[key].items.length > 0) {
      hasChanges = true;
      releaseNotes += `### ${categories[key].title}\n\n`;
      releaseNotes += categories[key].items.join('\n') + '\n\n';
    }
  }

  if (!hasChanges) {
    releaseNotes += 'No significant changes in this release.\n\n';
  }

  const changelogPath = path.join(__dirname, '..', 'CHANGELOG.md');
  let currentChangelog = '';
  
  if (fs.existsSync(changelogPath)) {
    currentChangelog = fs.readFileSync(changelogPath, 'utf8');
  } else {
    currentChangelog = '# Changelog\n\nAll notable changes to this project will be documented in this file.\n\n';
  }

  // Prevent duplicate entries for the same version
  if (currentChangelog.includes(`## [${currentVersion}]`)) {
    console.log(`Version ${currentVersion} already exists in CHANGELOG.md`);
    return;
  }

  // Insert release notes after the main header
  const headerMatch = currentChangelog.match(/^# Changelog[^\n]*\n+/i);
  if (headerMatch) {
    const header = headerMatch[0];
    const rest = currentChangelog.substring(header.length);
    fs.writeFileSync(changelogPath, header + releaseNotes + rest);
  } else {
    fs.writeFileSync(changelogPath, '# Changelog\n\n' + releaseNotes + currentChangelog);
  }

  console.log(`Successfully generated release notes for ${currentVersion}`);
}

generateChangelog();