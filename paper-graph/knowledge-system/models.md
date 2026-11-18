# Model inventory and routing

**Status:** observed 2026-11-19 on the MacBook session. Endpoints were queried; model lists are live. No model has been benchmarked for this project yet.

The important distinction: **authorized for this agent's subagents** is not the same as **configured in another client** or **served by the local llama.cpp router**. Only the first is guaranteed usable by the workflow and subagent tools.

## Route status (2026-11-19, T0.5)

| Bucket | Routes | What was proven |
|---|---|---|
| **Authorized** | `magpie/*`: deepseek-flash, auto/gemini-4-carbon, kimi/k3.1-pro, devin/glm-5-3, auto/glm-5.3, devin/swe-2 (+lightning). `magpie-anthropic/*`: zcode/GLM-5.3, devin/claude-opus-5-5 | one real two-token child call each, returning the expected text; catalog contexts and effort ladders read from the live config |
| **Local** | gemma4-12b, gemma4-e4b, qwen3.5-4b, glm-ocr, bge-m3 (llama.cpp router) | endpoint reachable, five models listed; unloaded models cold-start on demand. **Structured-output validity unmeasured** |
| **Unverified** | Claude Code and Codex client aliases; any route not listed above; `devin/swe-2` chat/JSON behaviour; no published per-token cost for the GLM routes | configured in a client, or listed in a catalog, but never called by this harness |

An *authorized* row means the transport works. It does not mean the model is good at the task: T0.6 measures schema validity and quote recovery, and nothing here replaces that.

## Authorized for DSH subagents

**Updated 2026-11-19 ~03:40** (before the subagent allowlist patch the only route was `deepseek-official`; that provider is now gone from discovery and Magpie took its place).

| Provider | Models allowed as child routes | Notes |
|---|---|---|
| magpie | deepseek/deepseek-flash, deepseek/deepseek-v4-pro, auto/gemini-4-carbon, auto/gemini-4-argon, auto/claude-opus-5-5, auto/glm-5.3, kimi/k3.1-pro(+highspeed), devin/glm-5-3, devin/kimi-k3-1-pro, devin/gemini-4-carbon, devin/swe-2 (+lightning/flash), cursor/gemini-4-carbon/argon | openai-completions wire; Magpie translates |
| magpie-anthropic | devin/claude-opus-5-5 (+fast), devin/claude-sonnet-5-5, devin/claude-fable-5-2 (+fast), `zcode/GLM-5.3` | anthropic-messages wire |

Verified by real child calls on 2026-11-19: `magpie/devin/glm-5-3`, `magpie/auto/glm-5.3` and `magpie-anthropic/zcode/GLM-5.3` each returned a correct two-token reply. The earlier 429 on ZCode was quota exhaustion; after the re-login the same id serves. `magpie/zcode/GLM-5.3` is not in the openai-completions allowlist — zcode is callable only through magpie-anthropic.

**Consequence:** a second family is now actually callable. GLM-5.3 (Zhipu) is independent of DeepSeek, Google, Anthropic and Moonshot, so it can review a DeepSeek or Gemini artefact. What it cannot do is review itself: `zcode/GLM-5.3`, `devin/glm-5-3` and `auto/glm-5.3` are the same model behind different gateways and are not independent of each other.

## Magpie dsh-tui catalog

Configured in ~/.dsh/profiles/dsh-tui/cordis.patch.yml and ~/.dsh/settings.yaml. The credential ref is MAGPIE_GATEWAY_KEY, supplied through ~/.dsh/.env; a refs entry in .credentials.yaml did not resolve for dsh 0.2.2-rc.3, while the .env fallback did.

| Route | Wire | Models | Role |
|---|---|---|---|
| magpie | openai-completions | Gemini Carbon/Argon, DeepSeek V4 Pro/Flash, Kimi K3.1, Devin SWE-2, devin/glm-5-3, routing groups (incl. auto/glm-5.3) | General route; Magpie translates. zcode is *not* here |
| magpie-anthropic | anthropic-messages | devin/claude-fable-5-2 (+fast), claude-opus-5-5 (+fast), claude-sonnet-5-5, claude-opus-4.7/4.8, **zcode/GLM-5.3** | Anthropic family plus the zcode GLM route; Magpie's generated catalog re-adds zcode/GLM entries after manual removal, so treat the split as generated, not curated |
| magpie-responses | openai-responses | grok/grok-4.7/4.6/4.5 family | Grok only |

Verified calls on 2026-11-19: magpie/deepseek-flash, magpie/auto/gemini-4-carbon, magpie/kimi/k3.1-pro, magpie/devin/swe-2, magpie/devin/glm-5-3, magpie/auto/glm-5.3 and magpie-anthropic/devin/claude-opus-5-5 returned OK. **Update ~03:45:** magpie-anthropic/zcode/GLM-5.3 also returned OK after the ZCode re-login. The earlier 429 ("ZCode: exceed quota limit") was quota exhaustion, not a bad id. zcode is not in the openai-completions allowlist, so it is reachable only through magpie-anthropic.

Caveat: the catalog is generated and updated by Magpie; a manual edit to the dsh-tui patch can be overwritten. `zcode/GLM-5.3-Flash` and `zcode/GLM-5.3` reappear under magpie-anthropic after Magpie writes settings, so the Anthropic-only split needs a post-generation override, not a hand edit. For **subagents**, use `magpie-anthropic` for ZCode GLM-5.3: `magpie/zcode/GLM-5.3` is refused by the child-route allowlist (measured 2026-11-19), even though the model id is recognized on the openai-completions wire. The allowlist, not the catalog, decides what a child call may use.

## Local llama.cpp router

Endpoints: http://192.168.50.18:8080/v1 (LAN) and https://llama.gw.emliunix.cc/v1 (public). Both returned the same five models. Router build b10107, max_instances 4, autoload on. bge-m3 was sleeping; the other four were unloaded and load on demand.

| Model | Modalities | Context | Quantization / device | Role |
|---|---|---|---|---|
| bge-m3 | text | 8192 (meta) | Q8_0, device none, 566M params, 1024-dim | Embeddings only |
| gemma4-12b | text, image, audio | 40960 (preset) | Q4_K_XL + MTP draft, ROCm0, reasoning off | Local extractor or second-family reviewer candidate |
| gemma4-e4b | text, image, audio | default (ctx arg 0) | Q4_K_XL, ROCm0 | Cheap triage, bulk classification |
| glm-ocr | text, image | default (ctx arg 0) | F16, device none | OCR fallback only; untested on PL notation |
| qwen3.5-4b | text, image | 16384 (preset) | BF16, ROCm0 | Cheap triage, classification, query rewriting |

No reranker is served. Hybrid search is reciprocal-rank fusion, not a cross-encoder rerank.

Cold start matters: unloaded models pay a load cost on first request. Measure time-to-first-token and tokens/s before using one for a 15-paper or 45-agent run.

## Nowledge Mem provider

The local Nowledge Mem config uses an OpenAI-compatible provider:

| Provider | Model | Max tokens | Timeout | Temperature |
|---|---|---|---|---|
| openai_compatible (zhipu) | glm-4.7 | 8192 | 60 s | 0.7 |
| openrouter | openai/gpt-5-mini | 8192 | 60 s | 0.7 |
| zai | zai/glm-4.7 | 8192 | 60 s | 0.7 |

This is the same 8192-token / 60-second configuration that failed earlier when a 16,384-token output budget was requested: 16 of 17 KG extraction attempts failed, and reasoning consumed the output budget. Before any nmem extraction backfill, set max_tokens >= output_budget and timeout >= output_budget / measured tokens-per-second, or reduce the budget. Do not ask for a 16K structured output on a 60 s / 8K route.

The config contains credentials. They are not reproduced here; keep them out of prompts, reports, and commits.

## Other clients (configured, not authorized here)

- **Claude Code 2.1.326** has model settings for many aliases across deepseek, kimi, zcode/GLM, cursor/Gemini, devin/GPT-6, grok, and auto routes. These are client settings, not proof that DSH subagents can call them. Treat them as candidates for a second family only after a smoke test.
- **Codex CLI 0.153.4** is configured with gpt-5.5, medium reasoning. Same caveat.
- The model list in a client config can include stale or unreachable aliases. Verify with a real call before counting a route as available.

## Recommended role matrix

| Role | Primary | Second family | Rule |
|---|---|---|---|
| Constitution, ontology, evaluation, paper outline | auto/gemini-4-carbon (high) | devin/claude-opus-5-5 | Highest independent rank in the 2026-11-19 matrix; do not route these to flash |
| Bulk drafting, normalization, transcription | deepseek-flash | — | 32-index model; only mechanically checkable output |
| Spine audit, pre-mortem, long-context check | kimi/k3.1-pro | auto/gemini-4-carbon | Needs a different family from the writer |
| Cheap cross-family review (mechanical artefacts, fan-out review) | zcode/GLM-5.3 (magpie-anthropic) | devin/glm-5-3, then auto/glm-5.3 | Same GLM-5.3 model behind all three routes: they are not independent of each other. zcode is preferred while its quota holds |
| Docs-eval second family (T5.3) | zcode/GLM-5.3 | gemma4-12b (local, privacy lane) | Independent only against a DeepSeek or Google writer, not against another GLM route |
| Ten structural checks, coding | devin/claude-opus-5-5 | auto/gemini-4-carbon | Strongest measured TB4 row in the briefing |
| Extraction arm | Freeze one after T0.6 | — | Do not mix across U2/S2/P2; record model, effort, temperature, prompt hash |
| Extraction judge | Human | Different family from extractor | LLM judge is secondary; never the sole grade |
| Bulk triage / classification | qwen3.5-4b or gemma4-e4b | — | Not for gold, not for high-risk edges |
| Embeddings | bge-m3 | — | 1024-d; do not switch mid-experiment |
| OCR fallback | glm-ocr | — | Only for pages where native extraction fails; test PL notation first |
| Gold annotation | Human | — | Never a model |

## Model smoke test before the pilot

Run the same three tasks on each candidate extraction model:

1. A 2-page PL excerpt with a theorem, a definition, and a citation.
2. A schema with 5 relations and an evidence object requiring page/quote/offsets.
3. A deliberately ambiguous case (mention vs use vs introduce).

Measure: JSON-schema validity on first pass, empty/truncated rate, finish reason, exact quote recovery, tokens/s, time-to-first-token, cost, and cold-start time. Freeze the winner's exact snapshot, prompt, schema, and decoding before the pilot.

## Non-claims

- Local model quality on PL extraction is unknown; no benchmark has run.
- The Claude/Codex routes are not verified for this harness and may not be callable.
- The local router's context limits are preset values; measure the effective context before a long prompt.
- Nmem's model route is a separate system; its 8192/60 configuration must be fixed independently.

## Sources

- list_subagent_models (magpie, magpie-anthropic), queried 2026-11-19 ~03:40.
- http://192.168.50.18:8080/v1/models and https://llama.gw.emliunix.cc/v1/models, queried 2026-11-19.
- Local Nowledge Mem remote_llm.json, read with credentials redacted.
- Claude Code settings.json and Codex config.toml, read for model aliases only.
