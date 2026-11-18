# Check template

One file per structural check. A check exists only if it can turn red on a named defect.

| Field | Rule |
|---|---|
| `check_id` | `check_<subject>`, stable. |
| `lesson` | The lesson id this check enforces. One check may serve several lessons; list them all. |
| `type` | lint / fixture / preflight / gate / review. "review" means a human judgment with a written rubric. |
| `path` | Repo-relative file that implements it. |
| `command` | The exact command a CI job or a human runs. No interactive steps. |
| `pass_condition` | A predicate over the command's output, not a feeling. |
| `seeded_defect` | The deliberate break that must make it red, and the expected message. |
| `owner` | The person or role accountable; a check with no owner is a wish. |

---

## Example — sync count guard (worked)

```yaml
check_id: check_sync_count_guard
lesson: [L03, L02]
type: fixture
path: tests/check_sync_count_guard.rs
command: cargo test -p paper-graph --test check_sync_count_guard
pass_condition: exit 0 and stdout contains "derived rows unchanged: 3/3"
seeded_defect: comment out the non-metadata count assertion in sync_library; the test must fail with "derived rows changed: 3/0"
owner: w4-checks
```
