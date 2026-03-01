# Skills

All skills live in this directory and are symlinked tinto the plugin `/skills` directories that need them.

| Skill | Description |
| ----- | ----------- |
| [crux-gen](crux-gen/SKILL.md) | Bootstrap a Crux-based application |

## Creating a Skill

1. Create the skill directory in `skills/<skill-name>/`
2. Add `SKILL.md` and `references/` as normal
3. Symlink into each plugin that needs it:

```bash
ln -s ../../../skills/<skill-name> ./plugins/<plugin>/skills/<skill-name>
```

4. Verify the symlink resolves correctly:

```bash
head -3 ./plugins/<plugin>/skills/<skill-name>/SKILL.md
```

See [CONTRIBUTING.md](../CONTRIBUTING.md#shared-skills) for full details.
