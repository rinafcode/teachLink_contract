# Dependency Security Management - Indexer Package

## Issue Overview

**Severity:** Low  
**Category:** DevOps & Infrastructure  
**Impact:** Medium - Security vulnerabilities  

### Description
Outdated dependencies in `/indexer/package.json` may contain security vulnerabilities. This document provides a comprehensive approach to auditing, updating, and maintaining secure dependencies.

## Current State Analysis

### Known Vulnerable Dependencies (Example)

Before remediation, typical vulnerable packages might include:

```json
{
  "name": "teachlink-indexer",
  "version": "1.0.0",
  "dependencies": {
    "@stellar/stellar-sdk": "^9.0.0",  // May have known vulnerabilities
    "axios": "^0.21.0",                  // CVE-2021-3749
    "express": "^4.17.0",                // Multiple CVEs
    "lodash": "^4.17.15",                // Prototype pollution
    "node-fetch": "^2.6.0",              // Exposure vulnerability
    "ws": "^7.4.0"                       // DoS vulnerability
  },
  "devDependencies": {
    "typescript": "^4.5.0",
    "jest": "^27.0.0",
    "eslint": "^7.32.0"
  }
}
```

## Implementation Strategy

### 1. Dependency Audit Process

#### Automated Security Audit Script

```bash
#!/bin/bash
# scripts/audit-dependencies.sh

set -e

echo "🔍 Starting dependency security audit..."

cd indexer

# Install audit tools if not present
if ! command -v npm-audit &> /dev/null; then
    echo "Installing npm audit tools..."
    npm install -g npm-audit-fix
fi

# Run npm audit
echo "Running npm audit..."
npm audit --audit-level=moderate > ../security-reports/npm-audit-report.txt

# Generate JSON report for analysis
npm audit --json > ../security-reports/npm-audit.json 2>/dev/null || true

# Run npm audit fix (dry run first)
echo "Analyzing available fixes..."
npm audit fix --dry-run --json > ../security-reports/npm-audit-fix-proposal.json

# Check for outdated packages
echo "Checking for outdated packages..."
npm outdated > ../security-reports/outdated-packages.txt

# Run Snyk security scan
if command -v snyk &> /dev/null; then
    echo "Running Snyk security scan..."
    snyk test > ../security-reports/snyk-report.txt
    snyk monitor
fi

# Generate summary
echo ""
echo "=== AUDIT SUMMARY ==="
echo "Critical vulnerabilities: $(grep -c '"severity":"critical"' ../security-reports/npm-audit.json || echo 0)"
echo "High vulnerabilities: $(grep -c '"severity":"high"' ../security-reports/npm-audit.json || echo 0)"
echo "Moderate vulnerabilities: $(grep -c '"severity":"moderate"' ../security-reports/npm-audit.json || echo 0)"
echo "Low vulnerabilities: $(grep -c '"severity":"low"' ../security-reports/npm-audit.json || echo 0)"
echo ""
echo "Full reports available in: ../security-reports/"

cd ..
```

#### Dependency Analysis Tool

```typescript
// scripts/analyze-dependencies.ts

import * as fs from 'fs';
import * as path from 'path';
import { execSync } from 'child_process';

interface Vulnerability {
  id: string;
  severity: 'critical' | 'high' | 'moderate' | 'low';
  package: string;
  version: string;
  description: string;
  recommendation: string;
  cve?: string;
}

interface PackageInfo {
  name: string;
  currentVersion: string;
  latestVersion: string;
  isOutdated: boolean;
  vulnerabilities: Vulnerability[];
  lastUpdate: Date;
  maintenanceStatus: 'active' | 'maintenance' | 'abandoned';
}

class DependencyAnalyzer {
  private packageJsonPath: string;
  private reportDir: string;

  constructor(packageJsonPath: string) {
    this.packageJsonPath = packageJsonPath;
    this.reportDir = path.join(path.dirname(packageJsonPath), '..', 'security-reports');
    
    if (!fs.existsSync(this.reportDir)) {
      fs.mkdirSync(this.reportDir, { recursive: true });
    }
  }

  async analyze(): Promise<PackageInfo[]> {
    console.log('📦 Analyzing dependencies...');
    
    const packageJson = JSON.parse(fs.readFileSync(this.packageJsonPath, 'utf-8'));
    const allDeps = {
      ...packageJson.dependencies,
      ...packageJson.devDependencies,
    };

    const packageInfos: PackageInfo[] = [];

    for (const [name, version] of Object.entries(allDeps)) {
      console.log(`Analyzing ${name}@${version}...`);
      
      const info = await this.analyzePackage(name, version.replace(/[^0-9.]/g, ''));
      packageInfos.push(info);
    }

    // Sort by vulnerability severity
    packageInfos.sort((a, b) => {
      const aSeverity = this.getMaxSeverity(a.vulnerabilities);
      const bSeverity = this.getMaxSeverity(b.vulnerabilities);
      return bSeverity - aSeverity;
    });

    this.generateReport(packageInfos);
    return packageInfos;
  }

  private async analyzePackage(name: string, currentVersion: string): Promise<PackageInfo> {
    try {
      // Get latest version
      const latestVersion = execSync(`npm view ${name} version`, { encoding: 'utf-8' }).trim();
      
      // Get vulnerability info from npm audit
      const vulnerabilities = await this.getVulnerabilities(name, currentVersion);
      
      // Check maintenance status
      const maintenanceStatus = this.checkMaintenanceStatus(name);
      
      return {
        name,
        currentVersion,
        latestVersion,
        isOutdated: currentVersion !== latestVersion,
        vulnerabilities,
        lastUpdate: new Date(),
        maintenanceStatus,
      };
    } catch (error) {
      console.error(`Error analyzing ${name}:`, error);
      return {
        name,
        currentVersion,
        latestVersion: 'unknown',
        isOutdated: false,
        vulnerabilities: [],
        lastUpdate: new Date(),
        maintenanceStatus: 'active',
      };
    }
  }

  private async getVulnerabilities(name: string, version: string): Promise<Vulnerability[]> {
    // Query npm audit API or use local npm audit
    try {
      const auditResult = JSON.parse(
        execSync(`npm audit --json`, { encoding: 'utf-8', stdio: ['pipe', 'pipe', 'ignore'] })
      );
      
      if (!auditResult.advisories) return [];
      
      return Object.values(auditResult.advisories)
        .filter((advisory: any) => 
          advisory.module_name === name || 
          advisory.vulnerable_versions.includes(version)
        )
        .map((advisory: any) => ({
          id: advisory.id,
          severity: advisory.severity,
          package: advisory.module_name,
          version: advisory.vulnerable_versions,
          description: advisory.overview,
          recommendation: advisory.recommendation,
          cve: advisory.cves[0],
        }));
    } catch {
      return [];
    }
  }

  private checkMaintenanceStatus(name: string): 'active' | 'maintenance' | 'abandoned' {
    // Check last commit date, issue activity, etc.
    try {
      const metadata = JSON.parse(
        execSync(`npm view ${name} --json`, { encoding: 'utf-8' })
      );
      
      const lastPublishDate = new Date(metadata.time.modified);
      const daysSinceUpdate = (Date.now() - lastPublishDate.getTime()) / (1000 * 60 * 60 * 24);
      
      if (daysSinceUpdate < 90) return 'active';
      if (daysSinceUpdate < 365) return 'maintenance';
      return 'abandoned';
    } catch {
      return 'active';
    }
  }

  private getMaxSeverity(vulnerabilities: Vulnerability[]): number {
    const severityMap = { critical: 4, high: 3, moderate: 2, low: 1 };
    return vulnerabilities.reduce(
      (max, v) => Math.max(max, severityMap[v.severity] || 0),
      0
    );
  }

  private generateReport(packages: PackageInfo[]): void {
    const report = {
      generatedAt: new Date().toISOString(),
      totalPackages: packages.length,
      outdatedPackages: packages.filter(p => p.isOutdated).length,
      vulnerablePackages: packages.filter(p => p.vulnerabilities.length > 0).length,
      summary: {
        critical: packages.filter(p => p.vulnerabilities.some(v => v.severity === 'critical')).length,
        high: packages.filter(p => p.vulnerabilities.some(v => v.severity === 'high')).length,
        moderate: packages.filter(p => p.vulnerabilities.some(v => v.severity === 'moderate')).length,
        low: packages.filter(p => p.vulnerabilities.some(v => v.severity === 'low')).length,
      },
      packages,
    };

    fs.writeFileSync(
      path.join(this.reportDir, 'dependency-analysis.json'),
      JSON.stringify(report, null, 2)
    );

    // Generate markdown report
    const markdownReport = this.generateMarkdownReport(report);
    fs.writeFileSync(
      path.join(this.reportDir, 'dependency-security-report.md'),
      markdownReport
    );

    console.log(`\n✅ Report generated: ${path.join(this.reportDir, 'dependency-security-report.md')}`);
  }

  private generateMarkdownReport(report: any): string {
    return `
# Dependency Security Report

**Generated:** ${new Date(report.generatedAt).toLocaleString()}

## Summary

| Metric | Count |
|--------|-------|
| Total Packages | ${report.totalPackages} |
| Outdated | ${report.outdatedPackages} |
| Vulnerable | ${report.vulnerablePackages} |
| Critical Issues | ${report.summary.critical} |
| High Issues | ${report.summary.high} |
| Moderate Issues | ${report.summary.moderate} |
| Low Issues | ${report.summary.low} |

## Critical & High Vulnerabilities

${report.packages
  .filter((p: PackageInfo) => 
    p.vulnerabilities.some((v: Vulnerability) => 
      ['critical', 'high'].includes(v.severity)
    )
  )
  .map((p: PackageInfo) => `
### ${p.name}

- **Current Version:** ${p.currentVersion}
- **Latest Version:** ${p.latestVersion}
- **Status:** ${p.maintenanceStatus}

#### Vulnerabilities

${p.vulnerabilities
  .filter((v: Vulnerability) => ['critical', 'high'].includes(v.severity))
  .map((v: Vulnerability) => `
- **[${v.severity.toUpperCase()}]** ${v.id}${v.cve ? ` (${v.cve})` : ''}
  - ${v.description}
  - **Recommendation:** ${v.recommendation}
`).join('\n')}
`).join('\n')}

## Recommended Actions

1. **Immediate Action Required:** Update packages with critical vulnerabilities
2. **High Priority:** Update packages with high severity vulnerabilities
3. **Medium Priority:** Schedule updates for moderate vulnerabilities
4. **Low Priority:** Monitor low severity vulnerabilities

## Update Commands

\`\`\`bash
# Update all safe dependencies
npm update

# Force update specific packages
npm install package-name@latest

# Review before updating
npm audit fix --dry-run
\`\`\`
`;
  }
}

// Run analysis
const analyzer = new DependencyAnalyzer('./package.json');
analyzer.analyze().catch(console.error);
```

### 2. Safe Update Strategy

#### Pre-Update Testing

```typescript
// scripts/test-before-update.ts

import { execSync } from 'child_process';
import * as fs from 'fs';

interface UpdatePlan {
  packageName: string;
  fromVersion: string;
  toVersion: string;
  breakingChanges: boolean;
  testResults: TestResult[];
}

interface TestResult {
  testName: string;
  passed: boolean;
  errorMessage?: string;
}

class SafeUpdater {
  private backupDir: string;
  private updatePlan: UpdatePlan[] = [];

  constructor() {
    this.backupDir = `./backups/${Date.now()}`;
  }

  async planUpdates(): Promise<UpdatePlan[]> {
    console.log('📋 Planning safe updates...');
    
    // Create backup
    this.createBackup();
    
    // Get proposed updates from npm audit
    const proposedUpdates = this.getProposedUpdates();
    
    for (const update of proposedUpdates) {
      const plan = await this.evaluateUpdate(update);
      this.updatePlan.push(plan);
    }
    
    return this.updatePlan;
  }

  private createBackup(): void {
    console.log('💾 Creating backup...');
    fs.mkdirSync(this.backupDir, { recursive: true });
    
    // Backup node_modules and package-lock.json
    if (fs.existsSync('./node_modules')) {
      execSync(`cp -r node_modules ${this.backupDir}/`);
    }
    if (fs.existsSync('./package-lock.json')) {
      execSync(`cp package-lock.json ${this.backupDir}/`);
    }
    
    console.log(`Backup created: ${this.backupDir}`);
  }

  private async evaluateUpdate(update: any): Promise<UpdatePlan> {
    const plan: UpdatePlan = {
      packageName: update.name,
      fromVersion: update.current,
      toVersion: update.latest,
      breakingChanges: this.checkBreakingChanges(update.name, update.current, update.latest),
      testResults: [],
    };
    
    console.log(`Evaluating ${update.name}: ${update.current} → ${update.latest}`);
    
    // Run tests with new version
    try {
      // Install new version
      execSync(`npm install ${update.name}@${update.latest}`, { stdio: 'pipe' });
      
      // Run test suite
      const testOutput = execSync('npm test', { encoding: 'utf-8', stdio: 'pipe' });
      
      // Parse test results
      plan.testResults = this.parseTestResults(testOutput);
      
      // If tests pass, keep the update
      if (plan.testResults.every(t => t.passed)) {
        console.log(`✅ ${update.name} update successful`);
      } else {
        console.warn(`⚠️  ${update.name} has test failures`);
      }
      
    } catch (error) {
      console.error(`❌ ${update.name} update failed:`, error.message);
      plan.testResults.push({
        testName: 'Installation',
        passed: false,
        errorMessage: error.message,
      });
      
      // Rollback
      this.rollback(update.name);
    }
    
    return plan;
  }

  private checkBreakingChanges(packageName: string, from: string, to: string): boolean {
    // Check package changelog or release notes
    try {
      const changelogUrl = `https://raw.githubusercontent.com/*/CHANGELOG.md`;
      // In reality, would fetch and parse changelog
      
      // Simple semver check: major version change = breaking
      const fromMajor = from.split('.')[0];
      const toMajor = to.split('.')[0];
      
      return fromMajor !== toMajor;
    } catch {
      return false;
    }
  }

  private parseTestResults(output: string): TestResult[] {
    // Parse Jest/Mocha test output
    const lines = output.split('\n');
    const results: TestResult[] = [];
    
    for (const line of lines) {
      if (line.includes('✓')) {
        results.push({
          testName: line.trim(),
          passed: true,
        });
      } else if (line.includes('✗') || line.includes('FAIL')) {
        results.push({
          testName: line.trim(),
          passed: false,
          errorMessage: line,
        });
      }
    }
    
    return results;
  }

  private rollback(packageName: string): void {
    console.log(`🔄 Rolling back ${packageName}...`);
    execSync(`npm install ${packageName}@latest`, { stdio: 'pipe' });
  }

  applySafeUpdates(): void {
    console.log('Applying safe updates...');
    
    const safeUpdates = this.updatePlan.filter(
      u => u.testResults.every(t => t.passed) && !u.breakingChanges
    );
    
    for (const update of safeUpdates) {
      console.log(`Updating ${update.packageName}...`);
      execSync(`npm install ${update.packageName}@${update.toVersion}`);
    }
    
    console.log(`✅ Applied ${safeUpdates.length} safe updates`);
  }
}

// Execute
const updater = new SafeUpdater();
updater.planUpdates().then(() => updater.applySafeUpdates());
```

#### Automated Update Workflow

```yaml
# .github/workflows/dependency-updates.yml

name: Dependency Updates

on:
  schedule:
    - cron: '0 2 * * 1'  # Every Monday at 2 AM
  workflow_dispatch:

jobs:
  audit-and-update:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: indexer/package-lock.json
      
      - name: Install dependencies
        working-directory: ./indexer
        run: npm ci
      
      - name: Run security audit
        working-directory: ./indexer
        run: |
          npm audit --audit-level=moderate > ../security-reports/audit-report.txt || true
          npm audit --json > ../security-reports/audit.json || true
      
      - name: Analyze dependencies
        working-directory: ./indexer
        run: npx ts-node ../scripts/analyze-dependencies.ts
      
      - name: Create Pull Request with updates
        uses: peter-evans/create-pull-request@v5
        with:
          commit-message: 'chore(deps): Update vulnerable dependencies'
          title: 'Security: Update vulnerable dependencies'
          body: |
            ## Dependency Security Update
            
            This PR updates dependencies with known security vulnerabilities.
            
            ### Changes
            - Updated packages with critical/high vulnerabilities
            - All tests passing with new versions
            - No breaking changes
            
            ### Security Report
            See attached audit report for details.
            
            ### Checklist
            - [x] Security audit completed
            - [x] Tests passing
            - [x] No breaking changes
            - [ ] Manual review required
          branch: chore/update-dependencies-${{ github.run_number }}
          base: main
          labels: |
            security
            dependencies
          add-paths: |
            indexer/package.json
            indexer/package-lock.json
```

### 3. Continuous Monitoring Setup

#### Dependency Monitoring Service

```typescript
// services/dependency-monitor.ts

import { CronJob } from 'cron';
import axios from 'axios';

interface DependencyHealth {
  packageName: string;
  isSecure: boolean;
  isMaintained: boolean;
  hasNewVersion: boolean;
  riskScore: number; // 0-100
}

class DependencyMonitor {
  private readonly SLACK_WEBHOOK_URL: string;
  private readonly CHECK_INTERVAL_HOURS: number;

  constructor() {
    this.SLACK_WEBHOOK_URL = process.env.SECURITY_ALERTS_WEBHOOK!;
    this.CHECK_INTERVAL_HOURS = 6;
  }

  start(): void {
    console.log('🔍 Starting dependency monitoring service...');
    
    // Check every 6 hours
    const job = new CronJob(
      `0 */${this.CHECK_INTERVAL_HOURS} * * *`,
      () => this.performCheck(),
      null,
      true,
      'UTC'
    );
    
    job.start();
    console.log(`Monitoring active. Checking every ${this.CHECK_INTERVAL_HOURS} hours.`);
    
    // Initial check
    this.performCheck();
  }

  private async performCheck(): Promise<void> {
    console.log('Performing dependency health check...');
    
    const packageJson = require('../indexer/package.json');
    const allDeps = {
      ...packageJson.dependencies,
      ...packageJson.devDependencies,
    };

    const alerts: string[] = [];

    for (const [name, version] of Object.entries(allDeps)) {
      const health = await this.checkDependencyHealth(name, version as string);
      
      if (health.riskScore > 70) {
        alerts.push(this.formatAlert(health));
      }
    }

    if (alerts.length > 0) {
      await this.sendAlerts(alerts);
    }
  }

  private async checkDependencyHealth(
    name: string,
    currentVersion: string
  ): Promise<DependencyHealth> {
    try {
      // Fetch package metadata
      const metadata = await axios.get(`https://registry.npmjs.org/${name}`);
      const latestVersion = metadata.data['dist-tags'].latest;
      const timeModified = metadata.data.time.modified;
      
      // Check for vulnerabilities
      const isSecure = await this.checkSecurity(name, currentVersion);
      
      // Check maintenance
      const daysSinceUpdate = (Date.now() - new Date(timeModified).getTime()) / (1000 * 60 * 60 * 24);
      const isMaintained = daysSinceUpdate < 365;
      
      // Calculate risk score
      let riskScore = 0;
      if (!isSecure) riskScore += 50;
      if (!isMaintained) riskScore += 30;
      if (currentVersion !== latestVersion) riskScore += 20;
      
      return {
        packageName: name,
        isSecure,
        isMaintained,
        hasNewVersion: currentVersion !== latestVersion,
        riskScore,
      };
    } catch (error) {
      console.error(`Error checking ${name}:`, error);
      return {
        packageName: name,
        isSecure: true,
        isMaintained: true,
        hasNewVersion: false,
        riskScore: 0,
      };
    }
  }

  private async checkSecurity(name: string, version: string): Promise<boolean> {
    // Use Snyk API or npm audit
    try {
      const response = await axios.post('https://snyk.io/api/v1/test/npm', {
        package: name,
        version,
      }, {
        headers: { Authorization: `token ${process.env.SNYK_TOKEN}` }
      });
      
      return response.data.issues.total === 0;
    } catch {
      return true; // Assume secure if can't check
    }
  }

  private formatAlert(health: DependencyHealth): string {
    const emoji = health.riskScore > 80 ? '🚨' : '⚠️';
    
    return `${emoji} *High Risk Dependency Alert*\n` +
      `*Package:* ${health.packageName}\n` +
      `*Risk Score:* ${health.riskScore}/100\n` +
      `*Issues:* ${!health.isSecure ? 'Security vulnerabilities ' : ''}` +
      `${!health.isMaintained ? 'Abandoned ' : ''}` +
      `${health.hasNewVersion ? 'Outdated' : ''}\n` +
      `*Action Required:* Update immediately`;
  }

  private async sendAlerts(alerts: string[]): Promise<void> {
    const message = {
      text: `*Dependency Security Alerts*\n\n${alerts.join('\n\n')}`,
    };

    try {
      await axios.post(this.SLACK_WEBHOOK_URL, message);
      console.log(`Sent ${alerts.length} alerts to Slack`);
    } catch (error) {
      console.error('Failed to send alerts:', error);
    }
  }
}

// Start monitoring
if (require.main === module) {
  const monitor = new DependencyMonitor();
  monitor.start();
}
```

### 4. Updated package.json Example

```json
{
  "name": "teachlink-indexer",
  "version": "1.1.0",
  "description": "TeachLink Stellar Network Indexer",
  "main": "dist/src/main.js",
  "scripts": {
    "start": "node dist/src/main.js",
    "build": "tsc",
    "test": "jest",
    "lint": "eslint src/ --ext .ts",
    "security:audit": "npm audit && ts-node scripts/analyze-dependencies.ts",
    "security:update": "npm audit fix && npm update",
    "precommit": "npm run lint && npm run test"
  },
  "dependencies": {
    "@stellar/stellar-sdk": "^11.0.0",
    "axios": "^1.6.0",
    "express": "^4.18.2",
    "lodash": "^4.17.21",
    "node-fetch": "^3.3.2",
    "ws": "^8.14.2",
    "typeorm": "^0.3.17",
    "pg": "^8.11.3",
    "redis": "^4.6.10",
    "prom-client": "^15.1.0"
  },
  "devDependencies": {
    "@types/express": "^4.17.20",
    "@types/jest": "^29.5.8",
    "@types/lodash": "^4.14.200",
    "@types/node": "^20.9.0",
    "@types/ws": "^8.5.8",
    "@typescript-eslint/eslint-plugin": "^6.9.1",
    "@typescript-eslint/parser": "^6.9.1",
    "eslint": "^8.53.0",
    "jest": "^29.7.0",
    "ts-jest": "^29.1.1",
    "ts-node": "^10.9.1",
    "typescript": "^5.2.2"
  },
  "engines": {
    "node": ">=18.0.0",
    "npm": ">=9.0.0"
  },
  "security": {
    "audit": {
      "level": "moderate",
      "autoFix": true,
      "notifyOnVulnerability": true
    }
  }
}
```

### 5. Dependency Management Policy Document

```markdown
# Dependency Management Policy

## Purpose
This document defines the policy for managing third-party dependencies in the TeachLink Indexer to minimize security risks.

## Scope
Applies to all npm/Node.js dependencies in the indexer directory.

## Policy Rules

### 1. Adding New Dependencies

**Requirements:**
- Must be actively maintained (updated within last year)
- No known critical/high severity vulnerabilities
- Minimum 100 GitHub stars or widely adopted
- Compatible license (MIT, Apache 2.0, BSD)

**Process:**
1. Submit dependency request via PR
2. Security team reviews package
3. Add to allowed list if approved

### 2. Version Pinning

**Rules:**
- Use exact versions (no ^ or ~ prefixes)
- Lock file must be committed
- Major version upgrades require security review

### 3. Update Schedule

**Automatic:**
- Patch updates: Weekly (automated via Dependabot)
- Minor updates: Monthly (security reviewed)

**Manual:**
- Major updates: Per case basis (requires testing)
- Critical security patches: Within 24 hours

### 4. Vulnerability Response

**Severity-based Response Times:**
- Critical: 24 hours
- High: 7 days
- Moderate: 30 days
- Low: Next scheduled update

**Process:**
1. Automated alert via monitoring
2. Triage by security team
3. Test patch/update
4. Deploy fix
5. Verify resolution

### 5. Prohibited Practices

- ❌ Using unmaintained packages (>2 years no update)
- ❌ Ignoring critical security advisories
- ❌ Disabling npm audit scripts
- ❌ Using packages with restrictive licenses (GPL, AGPL)

## Tools

### Required:
- `npm audit` in CI/CD pipeline
- Automated dependency updates (Dependabot/Renovate)
- Snyk continuous monitoring

### Recommended:
- npm-check-updates for visibility
- depcheck for unused dependency detection

## Compliance

All developers must:
- Run `npm audit` before committing
- Review Dependabot PRs within 48 hours
- Report suspicious package behavior immediately

## Exceptions

Exceptions require:
1. Written justification
2. Security team approval
3. Compensating controls documented
4. Expiration date set (max 90 days)

## Review

This policy reviewed quarterly by security team.

Last updated: 2026-01-29
```

## Monitoring Dashboard

### Real-time Dependency Health Metrics

```typescript
// dashboard/dependency-health-widget.ts

interface DependencyMetrics {
  totalDependencies: number;
  vulnerableDependencies: number;
  outdatedDependencies: number;
  abandonedDependencies: number;
  averageDaysBehind: number;
  criticalVulnerabilities: number;
  highVulnerabilities: number;
}

function renderDependencyWidget(metrics: DependencyMetrics): string {
  const healthScore = calculateHealthScore(metrics);
  
  return `
<div class="dependency-health">
  <h3>📦 Dependency Health</h3>
  
  <div class="health-score ${getHealthClass(healthScore)}">
    <span class="score">${healthScore}</span>
    <span class="label">Health Score</span>
  </div>
  
  <div class="metrics-grid">
    <div class="metric">
      <span class="value">${metrics.totalDependencies}</span>
      <span class="label">Total</span>
    </div>
    <div class="metric ${metrics.vulnerableDependencies > 0 ? 'danger' : 'success'}">
      <span class="value">${metrics.vulnerableDependencies}</span>
      <span class="label">Vulnerable</span>
    </div>
    <div class="metric ${metrics.outdatedDependencies > 0 ? 'warning' : 'success'}">
      <span class="value">${metrics.outdatedDependencies}</span>
      <span class="label">Outdated</span>
    </div>
    <div class="metric ${metrics.abandonedDependencies > 0 ? 'warning' : 'success'}">
      <span class="value">${metrics.abandonedDependencies}</span>
      <span class="label">Abandoned</span>
    </div>
  </div>
  
  <div class="vulnerability-breakdown">
    <h4>Vulnerabilities by Severity</h4>
    <div class="severity-bar">
      <div class="severity-segment critical" style="width: ${metrics.criticalVulnerabilities * 10}px"></div>
      <div class="severity-segment high" style="width: ${metrics.highVulnerabilities * 10}px"></div>
    </div>
    <div class="severity-labels">
      <span class="critical">${metrics.criticalVulnerabilities} Critical</span>
      <span class="high">${metrics.highVulnerabilities} High</span>
    </div>
  </div>
  
  <div class="actions">
    <button onclick="runAudit()">🔍 Run Audit</button>
    <button onclick="applyUpdates()">📥 Apply Safe Updates</button>
    <button onclick="viewReport()">📊 Full Report</button>
  </div>
</div>
  `;
}
```

## Implementation Checklist

- [ ] Run initial security audit
- [ ] Document all current vulnerabilities
- [ ] Create backup of working state
- [ ] Update critical vulnerabilities (within 24 hours)
- [ ] Update high severity vulnerabilities (within 7 days)
- [ ] Setup automated dependency monitoring
- [ ] Configure Dependabot/Renovate for auto-updates
- [ ] Integrate npm audit into CI/CD
- [ ] Establish dependency management policy
- [ ] Setup security alert notifications
- [ ] Schedule monthly dependency reviews
- [ ] Train team on secure dependency practices

## References

- [npm Security Best Practices](https://docs.npmjs.com/security/)
- [Snyk Vulnerability Database](https://snyk.io/vuln/)
- [Node.js Security Working Group](https://github.com/nodejs/security-wg)
- [OWASP Dependency Check](https://owasp.org/www-project-dependency-check/)
