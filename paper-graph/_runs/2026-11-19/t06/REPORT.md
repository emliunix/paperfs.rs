# T0.6 structured-output reliability

**No arm is frozen. W7 stays blocked pending human review.** The requested subagent transport could not be used at delegation depth 1 (remaining depth 0); these are direct Magpie gateway calls, so exact subagent behavior remains unverified.

Design: 2 routes × 3 cases × N=5 = 30 scored trials, effort low, 16,384 output tokens, 180 s/attempt, retry budget 0–2. Scores and latency use the first attempt; retries are observational and cannot rescue a failure. Six concurrent route/case streams ran trials in order. Source: paper 3154 pages 1 and 20. Structural means bare JSON plus full schema validity; semantic additionally requires every quote, code-point offset, page, and line locator to resolve.

| Route | Case | Structural | Semantic | Empty/truncated | p50 ms | p95 ms |
|---|---|---:|---:|---:|---:|---:|
| deepseek-flash | theorem/def/cite | 0/5 | 0/5 | 5/0 | 59452 | 95556 |
| deepseek-flash | five relations | 2/5 | 2/5 | 2/1 | 63882 | 81461 |
| deepseek-flash | mention vs use | 5/5 | 4/5 | 0/0 | 60970 | 101758 |
| glm-5.3 | theorem/def/cite | 0/5 | 0/5 | 5/5 | 160804 | 166266 |
| glm-5.3 | five relations | 0/5 | 0/5 | 5/5 | 143970 | 156117 |
| glm-5.3 | mention vs use | 0/5 | 0/5 | 5/1 | 180004 | 180022 |

| Route | Structural total | Semantic total | p50 ms | p95 ms |
|---|---:|---:|---:|---:|
| deepseek-flash | 7/15 | 6/15 | 60714 | 101758 |
| glm-5.3 | 0/15 | 0/15 | 160804 | 180022 |

## Failure taxonomy and decision

- **deepseek-flash:** {"empty": 7, "http_error": 7, "json_parse_error": 1, "semantic_line_locator_mismatch": 1, "semantic_offset_mismatch": 1, "truncated": 1}
- **glm-5.3:** {"empty": 15, "timeout": 4, "truncated": 11}
- Semantic pass is evidence recoverability only, not extraction correctness or human gold agreement.
- Four completed unscored probe attempts are retained under probe-4k/ and probe-16k/; two wrapper-aborted probe calls are logged in probe-aborts.json. Retry-only work stopped after deterministic token exhaustion/timeouts; missing trials resumed with retry budget 0. Scored failures and verbatim gateway errors are under attempts/.
- Freezing requires human review and, if strict protocol adherence is required, rerunning through the parent agent's subagent tool. Until then neither reliability nor latency qualifies an extraction arm.

## Post-hoc transport check (2026-11-19 12:35)

Two depth-0 subagent calls returned the requested JSON immediately: `magpie/deepseek/deepseek-flash` -> {"ok":true,"route":"probe"} and `magpie-anthropic/zcode/GLM-5.3` -> {"ok":true,"route":"probe"}. The subagent transport is therefore functional at depth 0; the T0.6 numbers above were produced by direct HTTP and remain transport-limited. A structured-output re-run through the subagent route is in progress; this report is unchanged until it lands.

## Grounded depth-0 probe (2026-11-19 12:52)

Both routes returned schema-valid JSON through the real subagent transport, and every reported quote/offset was verified against the case excerpt:

| Route | Items | Quote substrings | Offsets exact |
|---|---:|---|---|
| deepseek-flash | 3 | 3/3 | 3/3 |
| zcode/GLM-5.3 | 3 | 3/3 | 3/3 |

Offsets 1369, 6574, 6946, 7548 all matched `excerpt_text.indexOf(quote)` exactly. Therefore the T0.6 direct-HTTP results above are a transport artifact, not a model verdict. This is **not** a full re-run (1 case, 1 trial per route) and no arm is frozen; a bounded depth-0 re-run (3 trials per route) is the next measurement before any freeze recommendation.
