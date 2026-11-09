# Right panel redesign, revision 2

Status: candidate A approved for all three platforms; implementation in progress. The user rejected the first right-panel preview and requested frontend-design, then explicitly chose A and requested implementation. This revision supersedes the original Configuration | Activity arrangement only; surrounding runtime/table geometry stayed fixed in the design comparison.

## Diagnosis from the rejected screenshot

1. No primary reading path: profile identity, engine fields, activity, totals, timestamps and explanatory paragraphs have almost equal weight.
2. Labels and numeric values are separated by entire half-panel widths. For example, the old preview's Switches value ends at cell 196 while its label starts at 150, a 46-cell scan. Token labels/values span the same distance.
3. Last used, Recorded at, UTC offset and the activation list repeat one time concept in several places. Unset explanations and error-policy prose are visible even in a healthy state.
4. Usage is two disconnected tables plus a three-number strip. There is no visual comparison of the five existing token categories, and the full-width horizontal gaps make them feel unrelated.
5. All-cap headings, repeated blue accents and identical full-width rules compete with the actual values. The previous panel reads like a field dump.

## Design direction

Configuration inspector with a usage ledger. The distinctive element is a row-aligned terminal bar comparison of token magnitudes; everything around it stays quiet. This is appropriate to the subject because the user compares configuration and provider usage, not marketing claims or animated dashboards.

Tokens: existing canvas #1e1e2e; quiet inset #181825; separators #585b70; primary #cdd6f4; secondary #a6adc8; Codex accent #89b4fa (Claude/Grok retain their own identity tokens). Success/error/cost retain existing semantic palette roles. Terminal-owned monospace, normal/bold only, same cell height. No image-only larger fonts or rounded web cards that Ratatui cannot reproduce.

Titles use sentence case. Profile name is the visual anchor; status is short and aligned with it. One secondary description line. Model, endpoint and auth form one connection summary; tags sit with identity. Last activation appears once with date/time/offset. A one-line scope note explains activation meaning; hypothetical empty/error policies appear only when relevant.

## Candidates built at the same size

```text
A / Inspector, recommended          B / Split console
identity + status                   identity + status
model / connection summary          configuration | numeric ledger
provider totals                     activation    | category counts
aligned token bars + counts         compact notes | totals
activation history

C / Usage first
identity + status
provider totals + category ledger
configuration and auth
activation history
```

A follows the task's primary action, selecting and applying configurations, then gives provider consumption a strong visual form. B packs independent columns but can compress long endpoints/auth labels. C emphasizes consumption before configuration; it is more suitable when inspecting spending than switching profiles. All use the same fixture facts, surrounding shell, font, palette and terminal dimensions; preference is about order and density, not invented data.

## Data and graph rules

- Bar scale is the largest displayed category count, shown in the header. Each bar is independent. Counts preserve the source's meanings; do not call the chart a sum-to-100% composition, a cache hit rate, quota or trend.
- Five rows: Input, Cache read, Cache write, Output and Reasoning. Same scale and track width, numeric counts aligned to one column immediately after the track. Small nonzero values use partial block cells; zero stays an empty track. No minimum full-cell fake magnitude.
- Use only the existing ProviderBreakdownDto fields. Its `total_tokens` is independently reported; do not assume it equals the sum of token fields across all sources. Fixture arithmetic is not proof of upstream semantics (`crates/ccr-usage/src/queries.rs:119-152`).
- Usage scope and estimate qualifier remain visible in compact view: provider-level, all-time, approximate official-equivalent USD. No fake profit, cache saving or last API timestamp.
- Success-switch count stays beside activation history, not stranded at a far panel edge. Timezone is embedded in each exact timestamp or declared once for a coherent timeline.
- Top-three recorded activations are displayed as aligned rows; newest row is the last used value, not duplicated as another 'Recorded at' field. Empty history renders one no-record line.
- Keep configured fields/unknown/invalid semantics from the parent. Collapse unset optional configuration into expanded inspection; serious invalid/required auth issues remain visible in the normal view.

## Frozen verification matrix

Before delivery inspect three candidates at the same 107×32-cell panel, then the recommended layout on Codex/Claude/Grok. Exercise a 74-cell right rail and short 60×20 panel, a Chinese label version, missing usage/history, and long model/endpoint. The short panel must have explicit scrolling/reflow in the implementation contract, not silently omit content. This phase checks generated design artifacts; real Ratatui/terminal verification remains pending.

## Acceptance changes

- The first normal view identifies selected name/status, model, endpoint/auth and scope without reading body paragraphs.
- No separate UTC-offset field, duplicate last-used timestamp or prose describing how hypothetical failures behave in a healthy view.
- The five category rows share a value boundary and a common bar scale; source totals and estimates stay separate.
- Right-panel labels, connection fields and history maintain short horizontal association. No standalone single number at the extreme right of a 46-cell key/value row.
- Preserve current-vs-selected distinction, mask/sanitize secrets, zero-I/O rendering and original planned lifecycle safety.

## Review artifacts

- Recommended [A detail](right-v2-a.png) and [full Codex](codex-design-v2-a.png); [Claude](claude-design-v2-a.png) and [Grok](grok-design-v2-a.png) use the same hierarchy.
- Alternatives: [B split console](right-v2-b.png), [C usage first](right-v2-c.png).
- Edge cases: [narrow](right-v2-narrow.png), [short](right-v2-short.png), [Chinese](right-v2-zh.png), [empty](right-v2-empty.png), [long content](right-v2-long.png).
- Review result: A best supports selecting/applying profiles; B makes connection inspection and consumption compete across columns, while C postpones the configuration facts needed for the primary action. The user approved A. See [planning evidence](planning-validation.md) and [actual implementation evidence](visual-implementation.md).
