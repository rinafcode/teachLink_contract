# Issue Triage Guide

This document describes the process for triaging issues in the TeachLink repository. Effective triage ensures issues are properly categorized, prioritized, and assigned for efficient resolution.

---

## Triage Overview

### What is Triage?

Issue triage is the process of:
1. Reviewing new issues for completeness
2. Verifying and reproducing reported problems
3. Categorizing issues with appropriate labels
4. Prioritizing based on impact and urgency
5. Assigning to milestones and contributors

### Who Can Triage?

| Role | Triage Permissions |
|------|-------------------|
| Maintainers | Full triage authority |
| Core Contributors | Label and categorize |
| Contributors | Suggest labels in comments |
| Community | Report and comment |

---

## Triage Workflow

### Automated Workflow

```
New Issue Created
       │
       ▼
┌─────────────────┐
│ Auto-label:     │
│ "triage"        │
│ "needs-review"  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Assign to       │
│ triage project  │
└────────┬────────┘
         │
         ▼
  Awaits Manual
     Triage
```

### Manual Triage Steps

#### Step 1: Initial Review (< 2 days)

- [ ] Read the issue thoroughly
- [ ] Check if it's a duplicate
- [ ] Verify issue template was used correctly
- [ ] Request clarification if needed

#### Step 2: Verification

**For Bug Reports:**
- [ ] Reproduce the issue
- [ ] Verify on specified environment
- [ ] Check if already fixed in main

**For Feature Requests:**
- [ ] Assess alignment with roadmap
- [ ] Check for existing related features
- [ ] Evaluate feasibility

#### Step 3: Categorization

Apply appropriate labels:
- [ ] Type label (bug, enhancement, etc.)
- [ ] Priority label
- [ ] Component label
- [ ] Effort estimate

#### Step 4: Assignment

- [ ] Assign to milestone (if applicable)
- [ ] Assign to contributor (if volunteered)
- [ ] Remove "triage" label
- [ ] Add "confirmed" label

---

## Label System

### Type Labels

| Label | Description | Color |
|-------|-------------|-------|
| `bug` | Something isn't working | #d73a4a |
| `enhancement` | New feature or request | #a2eeef |
| `documentation` | Documentation improvements | #0075ca |
| `security` | Security related | #d93f0b |
| `question` | Further information requested | #d876e3 |
| `task` | Internal project task | #fbca04 |

### Priority Labels

| Label | Description | Color | Response Time |
|-------|-------------|-------|---------------|
| `priority: critical` | Immediate attention needed | #b60205 | < 24 hours |
| `priority: high` | Should be fixed soon | #d93f0b | < 3 days |
| `priority: medium` | Normal priority | #fbca04 | < 1 week |
| `priority: low` | Nice to have | #0e8a16 | < 2 weeks |

### Status Labels

| Label | Description | Color |
|-------|-------------|-------|
| `triage` | Needs initial review | #e4e669 |
| `needs-info` | Awaiting reporter response | #d876e3 |
| `confirmed` | Verified and accepted | #0e8a16 |
| `in-progress` | Being worked on | #1d76db |
| `blocked` | Waiting on dependency | #fef2c0 |
| `wontfix` | Will not be addressed | #ffffff |
| `duplicate` | Duplicate of another issue | #cfd3d7 |

### Component Labels

| Label | Description | Color |
|-------|-------------|-------|
| `contract: teachlink` | Main TeachLink contract | #5319e7 |
| `contract: insurance` | Insurance module | #5319e7 |
| `module: bridge` | Bridge functionality | #5319e7 |
| `module: escrow` | Escrow functionality | #5319e7 |
| `tooling` | Scripts and tools | #bfd4f2 |
| `ci/cd` | CI/CD pipeline | #bfd4f2 |

### Effort Labels

| Label | Description | Color |
|-------|-------------|-------|
| `effort: xs` | < 1 hour | #c5def5 |
| `effort: s` | 1-4 hours | #c5def5 |
| `effort: m` | 1-2 days | #c5def5 |
| `effort: l` | 3-5 days | #c5def5 |
| `effort: xl` | 1+ week | #c5def5 |

### Special Labels

| Label | Description | Color |
|-------|-------------|-------|
| `good first issue` | Good for newcomers | #7057ff |
| `help wanted` | Extra attention needed | #008672 |
| `breaking-change` | Would introduce breaking changes | #b60205 |

---

## Priority Guidelines

### Critical Priority

Assign `priority: critical` when:
- Security vulnerability discovered
- Data loss or corruption possible
- System completely unusable
- Financial impact on users

**Actions:**
1. Immediately notify maintainers
2. Create tracking incident
3. Begin work within 24 hours
4. Consider emergency release

### High Priority

Assign `priority: high` when:
- Major functionality broken
- No reasonable workaround exists
- Affects many users
- Blocks development progress

**Actions:**
1. Assign to current sprint
2. Begin work within 3 days
3. Provide regular updates

### Medium Priority

Assign `priority: medium` when:
- Feature works but has issues
- Workaround exists
- Moderate user impact
- Enhancement with clear value

**Actions:**
1. Add to backlog
2. Schedule for upcoming sprint
3. Consider for next release

### Low Priority

Assign `priority: low` when:
- Minor inconvenience
- Cosmetic issues
- Nice-to-have improvements
- Long-term enhancements

**Actions:**
1. Add to backlog
2. Mark as `good first issue` if appropriate
3. Review quarterly

---

## Triage Responses

### Templates

#### Requesting More Information

```markdown
Hi @{username}, thank you for reporting this issue!

To help us investigate, could you please provide:
- [ ] Steps to reproduce the issue
- [ ] Expected vs actual behavior
- [ ] Your environment details (OS, Rust version, CLI version)
- [ ] Relevant error messages or logs

I'll add the `needs-info` label for now and revisit once we have more details.
```

#### Confirming a Bug

```markdown
Thanks for the detailed report, @{username}! I've been able to reproduce this issue.

I'm marking this as confirmed and will add it to our backlog. We'll prioritize based on impact and available resources.

If you're interested in working on a fix, let us know and we can provide guidance!
```

#### Closing as Duplicate

```markdown
Hi @{username}, thanks for reporting this!

This appears to be a duplicate of #{issue_number}, which tracks the same problem. I'm closing this issue to consolidate the discussion there.

Please feel free to add any additional context to the original issue. Thanks for your understanding!
```

#### Closing as Won't Fix

```markdown
Hi @{username}, thank you for your suggestion!

After discussion, we've decided not to implement this because [reason].

We appreciate you taking the time to share your ideas. If you have any questions about this decision, feel free to open a discussion.
```

#### Marking as Good First Issue

```markdown
This looks like a great issue for newcomers! 

I've marked it as a `good first issue`. If you're new to the project and interested in contributing, check out our [Contributing Guide](CONTRIBUTING.md) for getting started.

Feel free to comment if you'd like to take this on!
```

---

## Special Cases

### Security Issues

When a security issue is reported:

1. **DO NOT** discuss details publicly
2. Verify it was reported through proper channels
3. If not, redirect to SECURITY.md
4. Apply `security` label
5. Notify security council privately
6. Follow security disclosure process

### Spam/Invalid Issues

1. Close immediately with brief explanation
2. Lock if necessary
3. Report to GitHub if violation of ToS

### Stale Issues

Issues with no activity for 30 days:
1. Add `stale` label
2. Comment asking for status
3. Close after 14 more days if no response

---

## Triage Meetings

### Weekly Triage

**When**: Every Monday, 15:00 UTC
**Where**: Discord #dev-chat voice channel
**Duration**: 30-60 minutes

**Agenda**:
1. Review new issues from past week
2. Reassess blocked issues
3. Validate priority assignments
4. Assign issues to sprints

### Triage Rotation

| Week | Primary | Backup |
|------|---------|--------|
| 1 | Maintainer A | Maintainer B |
| 2 | Maintainer B | Maintainer C |
| 3 | Maintainer C | Maintainer A |
| 4 | Maintainer A | Maintainer B |

---

## Metrics

### Triage Health Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Time to First Response | < 48 hours | Average time from creation to first comment |
| Triage Completion Rate | > 90% | % of issues triaged within 1 week |
| Backlog Age | < 30 days | Average age of untriaged issues |
| Stale Issue Rate | < 10% | % of issues going stale |

### Dashboard

Track metrics at: [GitHub Insights](https://github.com/rinafcode/teachLink_contract/pulse)

---

## Automation

### GitHub Actions Automation

The following is automated:
- Auto-labeling new issues with `triage`
- Stale issue management
- Duplicate detection (partial)
- Welcome message for first-time contributors

### Bot Commands

| Command | Action |
|---------|--------|
| `/label bug` | Add bug label |
| `/priority high` | Set high priority |
| `/assign @user` | Assign to user |
| `/duplicate #123` | Mark as duplicate |

---

## Questions?

- **Triage Questions**: Ask in #dev-chat
- **Process Improvements**: Open a discussion
- **Automation Issues**: Tag @maintainers

---

*Effective triage keeps our project healthy and contributors happy!*

---

*Last updated: January 2026*
