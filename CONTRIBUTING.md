# Contribution Guide

Augentic welcomes community contributions to the plugins repository.

Please familiarize yourself with this guide and [CLAUDE.md](CLAUDE.md) (architecture and authority hierarchy) before contributing.

There are many ways to help Augentic Plugins besides contributing code:

- Fix bugs or file issues
- Improve the documentation

## Table of Contents

- [Contribution Guide](#contribution-guide)
  - [Table of Contents](#table-of-contents)
  - [Contributing Code](#contributing-code)
  - [Plugin and Skill Guidelines](#plugin-and-skill-guidelines)
    - [Skill structure](#skill-structure)
    - [Shared skills and references](#shared-skills-and-references)
    - [Conventions](#conventions)
  - [Developer's Certificate of Origin](#developers-certificate-of-origin)
  - [Pull request procedure](#pull-request-procedure)
  - [Conduct](#conduct)

## Contributing Code

Unless you are fixing a known bug, we **strongly** recommend discussing it with the core team via a GitHub issue before getting started to ensure your work is consistent with the project's roadmap and architecture.

All contributions are made via pull request. Note that **all patches from all contributors get reviewed**. After a pull request is made other contributors will offer feedback, and if the patch passes review a maintainer will accept it with a comment. When pull requests fail testing, authors are expected to update their pull requests to address the failures until the tests pass and the pull request merges successfully.

At least one review from a maintainer is required for all patches (even patches from maintainers).

Reviewers should leave a "LGTM" comment once they are satisfied with the patch. If the patch was submitted by a maintainer with write access, the pull request should be merged by the submitter after review.

## Plugin and Skill Guidelines

Please follow these guidelines when contributing plugins and skills.

### Skill structure

Use [templates/skill-template.md](templates/skill-template.md) as the authoritative template. Every skill should include these sections in order: **Frontmatter** (YAML with `name`, `description`, `argument-hint`, `allowed-tools`), **Overview**, **Arguments** (or **Derived Arguments**), **Process**, **Reference Documentation**, **Examples**, **Error Handling**, **Verification Checklist**, **Important Notes**.

Place reference documentation in `references/`; place worked examples in `references/examples/` (when the skill has reference docs) or a top-level `examples/` directory (for standalone examples). Use numbered prefixes for example filenames (e.g., `01-simple-case.md`, `02-complex-case.md`).

### Shared skills and references

Skills used by multiple plugins live in `skills/` and are symlinked into each plugin. Share a skill when two or more plugins need it and its behavior is not plugin-specific. Reference documents used by multiple skills live in `references/` and are symlinked into each skill's `references/` directory.

### Conventions

- Keep SKILL.md files focused and concise; delegate detailed patterns to `references/`
- Prefer symlinks over copies; never duplicate a reference file that could be symlinked
- Use parameterized references (e.g., `$CHECKPOINT_FILENAME`) in shared docs so multiple skills can reference them
- Skills should be autonomous by default — avoid prompting the user for input during pipeline execution
- Use `[unknown]` tokens rather than guessing when information is ambiguous

When adding a new plugin, register it in `.claude-plugin/marketplace.json`. When modifying files in `references/` or `skills/`, review downstream consumers for compatibility and update SKILL.md links if section anchors change.

## Developer's Certificate of Origin

All contributions must include acceptance of the DCO:

```text
Developer Certificate of Origin
Version 1.1

Copyright (C) 2004, 2006 The Linux Foundation and its contributors.
660 York Street, Suite 102,
San Francisco, CA 94110 USA

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

To accept the DCO, simply add this line to each commit message with your name and email address (`git commit -s` will do this for you):

```text
Signed-off-by: Jane Example <jane@example.com>
```

For legal reasons, no anonymous or pseudonymous contributions are accepted. If this is an issue, please contact us via a [GitHub issue](https://github.com/augentic/plugins/issues).

## Pull request procedure

To make a pull request, you will need a GitHub account; if you are unclear on this process, see GitHub's documentation on [forking](https://help.github.com/articles/fork-a-repo) and [pull requests](https://help.github.com/articles/using-pull-requests). Pull requests should be targeted at the `main` branch. Before creating a pull request, go through this checklist:

1. Create a feature branch off of `main` so that changes do not get mixed up.
2. [Rebase](https://git-scm.com/book/en/Git-Branching-Rebasing) your local changes against the `main` branch.
3. Run `make validate` (or `make validate <plugin>`) and confirm the plugin validates.
4. Run `make checks` and confirm all documentation and link checks pass.
5. Accept the Developer's Certificate of Origin on all commits (see above).
6. Ensure that each commit has a subsystem prefix (e.g., `migrator:`, `crate-gen:`, `code-analyzer:`).

Pull requests will be treated as "review requests," and maintainers will give feedback on the style and substance of the patch.

When modifying shared references or skills used by multiple plugins, include a note on downstream compatibility and any SKILL.md updates required.

## Conduct

Whether you are a regular contributor or a newcomer, we care about making this community a safe place for you and we've got your back.

- We are committed to providing a friendly, safe and welcoming environment for all, regardless of gender, sexual orientation, disability, ethnicity, religion, or similar personal characteristic.
- Please avoid using nicknames that might detract from a friendly, safe and welcoming environment for all.
- Be kind and courteous. There is no need to be mean or rude.
- We will exclude you from interaction if you insult, demean or harass anyone. In particular, we do not tolerate behavior that excludes people in socially marginalized groups.
- Private harassment is also unacceptable. No matter who you are, if you feel you have been or are being harassed or made uncomfortable by a community member, please contact the Augentic maintainers immediately via a [GitHub issue](https://github.com/augentic/plugins/issues).
- Likewise any spamming, trolling, flaming, baiting or other attention-stealing behaviour is not welcome.

We welcome discussion about creating a welcoming, safe, and productive environment for the community. If you have any questions, feedback, or concerns please let us know with a [GitHub issue](https://github.com/augentic/plugins/issues).
