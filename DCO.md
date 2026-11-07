# Developer Certificate of Origin (DCO)

## Overview

Shimmy uses the Developer Certificate of Origin (DCO) to ensure that all contributions are properly licensed and that contributors have the right to submit their code.

## What is DCO?

The DCO is a lightweight way for contributors to certify that they wrote or otherwise have the right to submit their contribution. It's an industry-standard alternative to Contributor License Agreements (CLAs) used by projects like Linux kernel, Docker, and GitLab.

## DCO Text

By making a contribution to this project, I certify that:

```
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.

Everyone is permitted to copy and distribute verbatim copies of this
license document, but changing it is not allowed.

Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
    have the right to submit it under the open source license
    indicated in the file; or

(b) The contribution is based upon previous work that, to the best
    of my knowledge, is covered under an appropriate open source
    license and I have the right under that license to submit that
    work with modifications, whether created in whole or in part
    by me, under the same open source license (unless I am
    permitted to submit under a different license), as indicated
    in the file; or

(c) The contribution was provided directly to me by some other
    person who certified (a), (b) or (c) and I have not modified
    it.

(d) I understand and agree that this project and the contribution
    are public and that a record of the contribution (including all
    personal information I submit with it, including my sign-off) is
    maintained indefinitely and may be redistributed consistent with
    this project or the open source license(s) involved.
```

## How to Sign Your Commits

### Option 1: Set identity and explicitly sign off
Configure your identity, then use --signoff on each commit:

```bash
git config user.name "Your Name"
git config user.email "your.email@example.com"
```

Sign off each ordinary commit explicitly (format.signoff controls format-patch, not ordinary commits):
```bash
git commit --signoff -m "Add new feature"
```

### Option 2: Manual Sign-off
Add the `-s` flag to your git commit:

```bash
git commit -s -m "Add new feature"
```

This adds a sign-off line to your commit message:
```
Add new feature

Signed-off-by: Your Name <your.email@example.com>
```

### Option 3: Amend Existing Commits
If you forgot to sign off a commit:

```bash
# For the last commit
git commit --amend --signoff

# For multiple commits
git rebase --signoff HEAD~3  # Last 3 commits
```

## DCO Check

Maintainers verify DCO compliance for approved contributions. A configured DCO check may:

- ✅ **Pass**: All commits are properly signed off
- ❌ **Fail**: One or more commits are missing sign-off

If the DCO check fails:

1. **Add sign-off to missing commits** (see above)
2. **Force push the updated branch**:
   ```bash
   git push --force-with-lease origin your-branch
   ```
3. **Request maintainer re-review; do not assume an automatic check exists**

## Corporate Contributions

### For Individual Contributors
No additional setup required - just sign off your commits as described above.

### For Corporate Contributors
Organizations contributing to Shimmy should:

1. **Ensure developers can legally contribute** under their employment terms
2. **Have developers sign off commits** using their corporate email
3. **Consider additional CLA** for significant ongoing contributions (contact maintainer)

## Why DCO Instead of CLA?

**DCO Advantages:**
- **Lightweight**: No paperwork or legal review required
- **Standard**: Used by major projects (Linux, Docker, GitLab)
- **Developer-friendly**: Simple git sign-off process
- **Transparent**: All certifications are public in git history

**When CLA Might Be Needed:**
- Large corporate contributions
- Ongoing commercial partnerships
- Complex intellectual property situations

For such cases, contact the maintainer at `michaelallenkuykendall@gmail.com` to discuss a separate CLA.

## Frequently Asked Questions

### Q: What if I work for a company?
**A:** Use your work email and ensure your employer allows you to contribute to open source projects.

### Q: What if I contributed without signing off?
**A:** No problem! Amend your commits to add sign-off and force push the branch.

### Q: Is this legally binding?
**A:** Yes, the DCO is a legal certification that you have the right to contribute your code.

### Q: What about code I copied from elsewhere?
**A:** Only contribute code you wrote or have proper licensing rights to. When in doubt, ask the maintainer.

### Q: Can I sign off someone else's commit?
**A:** No, each contributor must sign off their own commits with their own identity.

## Enforcement

- **All commits** must be signed off to be accepted
- **Maintainer review** verifies sign-off for approved contributions; configured automation may assist
- **Maintainers** will not merge unsigned commits
- **No exceptions** - this protects both contributors and the project

## Contact

For questions about DCO or contribution licensing:
- **General questions**: Open a GitHub discussion
- **Corporate contributions**: Email `michaelallenkuykendall@gmail.com`
- **Legal concerns**: Email `michaelallenkuykendall@gmail.com`

---

*This DCO policy protects both contributors and users by ensuring all code contributions are properly licensed and legally contributed.*
