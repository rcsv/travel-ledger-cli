# Caglla.Travel 長期バージョン戦略

Caglla.Travel（CLI / 将来 Web）の **メジャーバージョンごとの到達イメージ** を整理したロードマップメモです。

**本書の位置付け:**

- 今後の設計議論・優先順位判断の **参考資料**
- **実装指示ではない** — スケジュール・スコープの確約でもない
- リポジトリの **パッチリリース**（v1.14.0、v1.16.0 等の documentation-only release）とは **別軸** の「製品メジャー版」想定

関連: [Travel Ledger Responsibilities](specifications/travel-ledger-responsibilities.md) / [Summary Implementation Plan](specifications/summary-implementation-plan.md) / [Reservation Implementation Plan](specifications/reservation-implementation-plan.md) / [data-model.md](data-model.md)

---

## 基本方針

メジャーバージョンは **技術的な区切り** ではなく、

```text
ユーザーが何をできるようになるか
```

で区切る。

---

## v1 — Planning Foundation

**現在の到達点（および v1 系で完結させたい基盤）。**

### 目的

```text
旅行計画を立てられる
旅行実績を記録できる
```

### 主な要素

```text
Trip
Day
Itinerary
Checklist
Expense
Note
Remark
Summary        ← v1.17.0 実装済み
Reservation    ← v1.18.0 実装済み
```

Summary と Reservation は **設計 → 実装（v1.17.0 / v1.18.0）まで完了**。v1 系 Hardening（v1.19–v1.22）の後、**Planning Foundation 完了総括** を [planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md) に文書化（**tag v1.23.0 は作らない** — v2.0.0 リリース後に landing）。

**v1 Planning Foundation** は v1.22 + Hardening 系列で実質完了。**製品 v2 Participant Foundation** は [v2.0.0](releases/v2.0.0-notes.md) でリリース済み。

### v1 系に持ち込まない想定（v1 クローズ時点 — 意図的 defer）

```text
Photo
Attachment
Participant          ← v2.0.0 で実装済み（本節は v1 当時の defer 記録）
Travel Journal（実装）
Budget / Settlement
```

v1 完了後、製品の次テーマは **v2 Participant Foundation**（§v2、**v2.0.0 リリース済み**）であった。v2 完了後の次テーマは **v3 Shared Expense**（§v3、**v3.0.0 リリース済み**）。v3.0.0 後の v3 系機能追加として **Estimate / Planned Budget** は **v3.1.0 リリース済み**（§v3.1）。

---

## v2 — Participant Foundation（**v2.0.0 リリース済み**）

### テーマ

```text
誰と旅行するか
```

### 実装内容（v2.0.0）

```text
Participant（Trip-scoped participation record）
```

リリースノート: [v2.0.0-notes.md](releases/v2.0.0-notes.md)

v2 の `participants` は **ある Trip への参加行**（TripParticipant-like）であり、**人そのものの正本ではない**。パスポート・生年月日・マイレージ等の Trip 横断プロフィールは、将来 Root スコープの **Person / Traveler Profile** として検討する（v2.0.0 では未実装）。詳細は [participant-model.md](specifications/participant-model.md#conceptual-model-person-vs-trip-participation)。

### スコープ

この段階では **精算機能は持ち込まない**。Root-level Person / Traveler Profile も **持ち込まない**。

例:

```text
父 / 母 / 妻 / 長男 / 次男
```

参加者情報を Trip に紐付けられることが目的（参加関係のレジストリ）。**Participant は自分を含む旅行参加者全員** を指す（Companion は自分以外）。人数統計では `participant_count` と `companion_count` を混同しない。v2.0.0 では `is_self` 列で「この Trip における自分」をマークする。詳細は [participant-model.md §Participant count semantics](specifications/participant-model.md#participant-count-semantics)。

設計系列（GitHub Epic #6）: [participant-model.md](specifications/participant-model.md)（#7）→ [participant-entity-design.md](specifications/participant-entity-design.md)（#8）→ 設計補正（#21 / #22）→ [participant-implementation-plan.md](specifications/participant-implementation-plan.md)（#9）→ Implementation（#10 / PR #24）→ [participant-post-implementation-review.md](specifications/participant-post-implementation-review.md)（#11）→ Release v2.0.0（#12）。

---

## v3 — Shared Expense（**v3.0.0 リリース済み**）

### テーマ

```text
誰が払ったか
誰の費用か
```

### 実装内容（v3.0.0）

```text
Paid By（structured payer）
Beneficiaries（equal split recording）
Export schema v5
```

リリースノート: [v3.0.0-notes.md](releases/v3.0.0-notes.md)

ここで **Expense が Participant と結び付く**。beneficiary 0 件 = personal、1 件以上 = shared（Settlement 計算 CLI は v3.x defer）。

設計系列（GitHub Epic #13）: [shared-expense-model.md](specifications/shared-expense-model.md)（#30）→ [shared-expense-entity-design.md](specifications/shared-expense-entity-design.md)（#31）→ [shared-expense-implementation-plan.md](specifications/shared-expense-implementation-plan.md)（#32）→ Implementation（#33 / PR #39）→ [shared-expense-post-implementation-review.md](specifications/shared-expense-post-implementation-review.md)（#34 / PR #40）→ Release v3.0.0（#35）。

### v3.x defer

```text
Settlement / transfer calculation
trip expense-summary
share_ratio / weighted split
--paid-by alias
```

---

## v3.1 — Estimate / Planned Budget（**v3.1.0 リリース済み**）

### テーマ

```text
予定費用（Planned Money）の構造化
Itinerary 配下の事前見積
Trip 全体の Planned total（Estimate 集計）
```

### 実装内容（v3.1.0）

```text
Estimate CRUD（Itinerary 配下）
src/money.rs（amount / currency 共通化）
Export schema v6（days[].itineraries[].estimates[]）
validate-export / trip diff（v6+）
trip stats Planned total
trip export-md 予定費用表示
itinerary replicate Estimate コピー
Post-Implementation Review（PR #57）
```

リリースノート: [v3.1.0-notes.md](releases/v3.1.0-notes.md)

**Planned Budget** は独立エンティティ **ではない**。Trip / Itinerary 単位の予定合計は **Estimate 行の derived 集計**。Actual Money は従来どおり Expense が正本。

設計系列: [estimate-model.md](specifications/estimate-model.md) → [estimate-entity-design.md](specifications/estimate-entity-design.md) → [estimate-implementation-plan.md](specifications/estimate-implementation-plan.md) → Phase 1–4（PR #50–#53）→ [estimate-post-implementation-review.md](specifications/estimate-post-implementation-review.md)（PR #57）→ Release v3.1.0。

### v3.1.x defer（Estimate 関連）

```text
Budget 独立エンティティ（Trip 全体予算上限）
Estimate payer / beneficiary / participant 連動
unit_amount × quantity
FX conversion
--without-estimates（replicate）
doctor / advisor での Estimate 活用
GUI / Web 版 Planned vs Actual 表示
```

v3.0.0 の Shared Expense defer（Settlement 等）は **引き続き v3.x defer**。

---

## v3.2 — Database Status（**v3.2.0 リリース済み**）

### テーマ

```text
参照中の caglla.db を明示する
サンプル DB・本番 DB・テスト DB の混同を防ぐ
```

### 実装内容（v3.2.0）

```text
db path（絶対パス表示、open なし）
db status / db status --json（存在確認、table counts、trip export schema version）
open 前 dispatch（未存在 DB を作成しない）
db reset 既存挙動維持
Post-Implementation Review（v3.2.0-db-status-post-implementation-review.md）
```

リリースノート: [v3.2.0-notes.md](releases/v3.2.0-notes.md)

設計系列: [v3.2.0-db-status-implementation-plan.md](specifications/v3.2.0-db-status-implementation-plan.md) → PR #58 → Post-Implementation Review → Release v3.2.0。

### v3.2.x defer

```text
--db <path> / CAGLLA_DB / db use（DB パス切替・永続設定）
doctor / advisor Estimate 活用
Trip / Itinerary command handler の commands/ 一括移動
```

---

## v3.2.1 — Module Layout（**v3.2.1 リリース済み**）

### テーマ

```text
src/ 内部構成の責務別整理
behavior-preserving refactor
```

### 実装内容（v3.2.1）

```text
cli/ commands/ domain/ storage/ analysis/ io/ output/
main.rs から Clap 定義を cli/args.rs へ
print_json を output::json へ集約
Post-Implementation Review（v3.2.1-module-layout-post-implementation-review.md）
```

リリースノート: [v3.2.1-notes.md](releases/v3.2.1-notes.md)

**挙動非変更** — CLI・JSON schema・DB schema・export/import 形式に変更なし。

設計系列: [v3.2.1-module-layout-implementation-plan.md](specifications/v3.2.1-module-layout-implementation-plan.md) → PR #59 → Post-Implementation Review → Release v3.2.1。

---

## v3.3 — Planned vs Actual Difference（**v3.3.0 リリース済み**）

### テーマ

```text
Trip 単位で予定（Estimate）と実績（Expense）の通貨別差分を表示する
```

### 実装内容（v3.3.0）

```text
Difference = Actual - Planned（通貨別、derived 集計）
trip stats human 出力に Difference:
trip stats --json に difference_totals（additive）
trip export-md Overview に Difference:
DB schema / export schema v6 変更なし
Post-Implementation Review（v3.3.0-planned-vs-actual-post-implementation-review.md）
```

リリースノート: [v3.3.0-notes.md](releases/v3.3.0-notes.md)

**表示 gate:** `estimate_count > 0 && expense_count > 0` のときのみ Difference を出す。Estimate のみ / Expense のみの Trip では省略。

設計系列: [v3.3.0-planned-vs-actual-implementation-plan.md](specifications/v3.3.0-planned-vs-actual-implementation-plan.md) → PR #60 → Post-Implementation Review → Release v3.3.0。

### v3.3.x defer

```text
Budget 独立エンティティ
doctor / advisor での Estimate 活用（v3.5.0 候補）
FX conversion
--db <path> / CAGLLA_DB / db use
trip export JSON への difference 永続化
Shared Expense / Settlement 連動
GUI / Web 版 Planned vs Actual カード表示
```

v3.0.0 の Shared Expense defer（Settlement 等）および v3.1.x の Budget / participant 連動 defer は **引き続き v3.x defer**。

---

## v3.4 — Itinerary-level Planned vs Actual Difference（**v3.4.0 リリース済み**）

### テーマ

```text
Itinerary 単位で予定（Estimate）と実績（Expense）の通貨別差分を export-md に表示する
```

### 実装内容（v3.4.0）

```text
Difference = Actual - Planned（Itinerary 内、通貨別、derived 集計）
gate = itinerary 内 estimate_count > 0 && expense_count > 0
trip export-md Itinerary セクションに Planned / Actual / Difference サマリー
DB schema / export schema v6 変更なし
trip stats / JSON への Itinerary 一覧は追加しない
小数通貨の負数表示 fix（format_amount_value — PR #62）
```

Post-Implementation Review: [v3.4.0-itinerary-planned-vs-actual-post-implementation-review.md](specifications/v3.4.0-itinerary-planned-vs-actual-post-implementation-review.md)

リリースノート: [v3.4.0-notes.md](releases/v3.4.0-notes.md)

設計系列: [v3.4.0-itinerary-planned-vs-actual-implementation-plan.md](specifications/v3.4.0-itinerary-planned-vs-actual-implementation-plan.md) → PR #61 / PR #62 → Post-Implementation Review → Release v3.4.0。

**Difference vs Settlement:** Difference は `Actual − Planned` のみ。事前入金・最終精算バランスは Balance / Settlement 別レイヤー（v3.4.0 非対象）。

### v3.4.x defer

```text
Day 単位 difference
trip stats human / --json への itinerary_differences
doctor / advisor Estimate 活用（v3.5.0 候補）
Budget 独立エンティティ
FX conversion
Balance / Settlement / advance payment / transfer
--db <path> / CAGLLA_DB / db use
commands/ への段階移行
```

---

## v3.5 — Receipt Inbox（**v3.5.0 リリース済み — documentation-only**）

### テーマ

```text
Itinerary に紐づける前の支払い証拠（未整理レシート）を一時保存し、
旅行後の振り返り・確認・Expense への昇格を支援する
```

### リリース内容（v3.5.0 — documentation-only）

```text
Receipt Inbox concept design を正式な v3.5.0 設計ドキュメントとして採用
Estimate = 予定 / Receipt = 未整理の証拠 / Expense = 確定した実績
Receipt Inbox は post-trip review 支援（旅行中の予算統制ではない）
Planned vs Actual の Actual には Receipt を直接含めない（補助表示のみ）
DB schema / export schema v6 変更なし / CLI 実装なし
```

Concept Design: [v3.5.0-receipt-inbox-concept-design.md](specifications/v3.5.0-receipt-inbox-concept-design.md)

リリースノート: [v3.5.0-notes.md](releases/v3.5.0-notes.md)

設計系列: [v3.5.0-receipt-inbox-concept-design.md](specifications/v3.5.0-receipt-inbox-concept-design.md) → Release v3.5.0（documentation-only）。

**Receipt vs Settlement:** Receipt は支払い証拠の整理。誰が払い・誰が負担するかは Balance / Settlement 別レイヤー（defer 継続）。

### v3.5.x defer（実装・拡張）

```text
Receipt image handling（image_path 先行実装・export/import・archive・CUI 表示は含む）
Day 単位 Planned vs Actual difference
doctor / advisor Estimate / Receipt 活用
Budget 独立エンティティ
FX conversion
Balance / Settlement / advance payment / transfer
generic Attachment / Photo model
--db <path> / CAGLLA_DB / db use
commands/ への段階移行
```

Receipt Inbox **実装**は [v3.6.0-receipt-inbox-metadata-only-implementation-plan.md](specifications/v3.6.0-receipt-inbox-metadata-only-implementation-plan.md) で **metadata-only から検討**（export schema v7 候補）。Receipt image は引き続き deferred。

---

## v3.6 — Receipt Inbox Metadata-only（**v3.6.0 リリース済み**）

### テーマ

```text
画像なしで Expense 化待ちの未整理支払い候補（Receipt）を Trip スコープで保存・整理する
```

### リリース内容（v3.6.0 — 実装リリース）

```text
receipts table（image_path なし、day_id optional のみ）
receipt add/list/show/update/ignore/delete
status: unreviewed / ignored のみ
export schema v7 — trip-level receipts[]
Planned vs Actual / trip stats / export-md には反映しない
v6 import 互換維持
```

Implementation Plan: [v3.6.0-receipt-inbox-metadata-only-implementation-plan.md](specifications/v3.6.0-receipt-inbox-metadata-only-implementation-plan.md)

Post-Implementation Review: [v3.6.0-receipt-inbox-metadata-only-post-implementation-review.md](specifications/v3.6.0-receipt-inbox-metadata-only-post-implementation-review.md)

リリースノート: [v3.6.0-notes.md](releases/v3.6.0-notes.md)

設計系列: [v3.5.0-receipt-inbox-concept-design.md](specifications/v3.5.0-receipt-inbox-concept-design.md) → Implementation Plan → 実装 → Post-Implementation Review → Release v3.6.0。

### v3.6.x defer（維持）

```text
receipt convert / promote（Expense 作成ロジック接続）
Receipt / Expense 共通の Evidence / Attachment 画像証憑
Receipt image handling / OCR / automatic receipt parsing
post-trip review auxiliary / Potential Actual 表示
Day 単位 Planned vs Actual difference
Balance / Settlement
trip stats への Receipt 反映
```

---

## v3.7 — Receipt Assignment / Trash workflow（**v3.7.0 リリース済み**）

### テーマ

```text
Receipt Inbox の user workflow を単純化し、
Receipt を「Expense 化（assign）」または「不要なので Trash」へ整理できるようにする
```

### 位置づけ

```text
v3.7.0 実装リリース済み（tag v3.7.0 @ 90e902a）
v3.7.1 patch リリース済み（tag v3.7.1 @ d498e70）— Okinawa Receipt Inbox sample + trashed receipt export fix
```

### リリース内容（v3.7.0 — 実装リリース）

```text
receipt assign（transaction 必須、Receipt 削除）
receipt trash / restore
receipt ignore は deprecated alias（trash 相当）
pending sum（receipt list に統合）
receipts.trashed_at、ignored → trashed migration
export schema v8（trashed_at）
v6 / v7 import 互換
Planned vs Actual / trip stats / export-md には反映しない
```

Workflow Design: [v3.7.0-receipt-assignment-and-trash-workflow-design.md](specifications/v3.7.0-receipt-assignment-and-trash-workflow-design.md)

Implementation Plan: [v3.7.0-receipt-assignment-and-trash-implementation-plan.md](specifications/v3.7.0-receipt-assignment-and-trash-implementation-plan.md)

リリースノート: [v3.7.0-notes.md](releases/v3.7.0-notes.md)

設計系列: Workflow Design → Implementation Plan → 実装（776bab6）→ Release v3.7.0（90e902a）。

### v3.7.1 — patch（**リリース済み**）

```text
Okinawa / Sesoko canonical sample に Receipt Inbox story を追加
trashed_at export を RFC3339 に整形（validate-export 互換）
export schema v8 変更なし / 新 CLI コマンドなし
```

リリースノート: [v3.7.1-notes.md](releases/v3.7.1-notes.md)

Post-Implementation Review: [v3.7.1-receipt-inbox-post-implementation-review.md](specifications/v3.7.1-receipt-inbox-post-implementation-review.md)

設計系列: v3.7.0 Release → Okinawa sample（feb4043）→ Release v3.7.1（d498e70）→ Post-Implementation Review。

設計系列: v3.7.0 Release → Okinawa sample（feb4043）→ Release v3.7.1（d498e70）→ Post-Implementation Review → **v3.8.0 roadmap realignment**（本書 §v3.8）。

### v3.7.x defer（維持）

```text
receipt purge
standalone receipt summary
Expense reassign / unassign / trash
Receipt / Expense 共通の Evidence / Attachment 画像証憑
Receipt image handling / OCR / automatic receipt parsing
Balance / Settlement
Potential Actual 表示 / Settlement warning
Planned vs Actual / trip stats への Receipt 反映
```

---

## v3.8 — Roadmap realignment（**v3.8.0 — documentation-only**）

### テーマ

```text
v3.7.1 後の製品メジャー版ロードマップ再整列
v4 Reservation 矛盾の解消
次候補テーマの開始可否整理
```

### リリース内容（v3.8.0）

```text
v4 = Reservation を退役（v1.18.0 実装済みと明記）
v4 = Travel Book に差し替え（旧 v5）
v5–v8 = Travel Journal / Identity / Cloud / Platform（旧 v6–v9 を繰り上げ）
Do not start yet を current-work と同期
コード / DB / export / CLI 変更なし
```

Roadmap document: [v3.8.0-roadmap-realignment-after-receipt-inbox.md](specifications/v3.8.0-roadmap-realignment-after-receipt-inbox.md)

リリースノート: [v3.8.0-notes.md](releases/v3.8.0-notes.md)

### v3.8 後の設計着手候補（実装は別判断）

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | DB path 切替（`--db` / `CAGLLA_DB` / `db use`） | **Phase 2 設計完了（v3.10.0）** | Phase 1 **v3.9.0**；Phase 2 **`db use` v3.11.0** |
| 中 | Travel Book v4 concept design | **v4.0.0 完了** | v4.1+ 実装候補 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |

---

## v3.9 — Config and DB path foundation（**v3.9.0**）

### テーマ

```text
DB 参照先の明示的解決（--db / CAGLLA_DB / caglla.toml）
v3.2.0 db path / db status の拡張
```

### リリース内容（v3.9.0 Phase 1）

```text
--db <path>（global）
CAGLLA_DB
./caglla.toml [database].path
default ./caglla.db
db status JSON schema v2（path_source / config_path）
SQLite schema / trip export schema 変更なし
```

Implementation plan: [v3.9.0-config-and-db-path-foundation-implementation-plan.md](specifications/v3.9.0-config-and-db-path-foundation-implementation-plan.md)

リリースノート: [v3.9.0-notes.md](releases/v3.9.0-notes.md)

### v3.9 後の設計着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 中 | DB path Phase 2（`db use` 実装） | **v3.10.0 設計完了** | **v3.11.0** |
| 低 | 親 dir 探索 / user-global config | defer | 未 |
| 中 | Travel Book v4 concept design | **v4.0.0 完了** | v4.1+ 実装候補 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |

**着手不可（canonical defer）:** Evidence / Attachment / image_path / OCR / Balance / Settlement / Travel Journal 実装 / Receipt→Actual 集計変更 等 — [current-work.md](current-work.md) を正本とする。

---

## v3.10 — DB Use concept design（**v3.10.0 — documentation-only**）

### テーマ

```text
db use — caglla.toml への [database].path 永続化（Phase 2 設計）
v3.9.0 read resolution の write 側
```

### リリース内容（v3.10.0）

```text
db use / db use --clear の責務・保存先・TOML 更新ルール
相対/絶対 path 保存、未知キー保持、コメント非保証
親 dir 探索は Phase 2 以降 defer
コード / DB / export / CLI 変更なし
```

Concept design: [v3.10.0-db-use-concept-design.md](specifications/v3.10.0-db-use-concept-design.md)

リリースノート: [v3.10.0-notes.md](releases/v3.10.0-notes.md)

### v3.10 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | `db use` Implementation Plan + Phase 2 実装 | **v3.10.0 完了** | **v3.11.0** |
| 中 | Travel Book v4 concept design | **v4.0.0 完了** | v4.1+ 実装候補 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |
| 低 | 親 dir config 探索 | defer Phase 3+ | 未 |

---

## v3.11 — DB Use implementation（**v3.11.0**）

### テーマ

```text
db use — CWD ./caglla.toml への [database].path 永続化（Phase 2 実装）
v3.10.0 concept design の実装
```

### リリース内容（v3.11.0）

```text
caglla db use <path> / caglla db use --clear
相対パス優先保存、未知 TOML キー保持、atomic write
db path / db status / --db / CAGLLA_DB 優先順位は v3.9.0 維持
SQLite schema / trip export schema 変更なし
```

Implementation plan: [v3.11.0-db-use-implementation-plan.md](specifications/v3.11.0-db-use-implementation-plan.md)

リリースノート: [v3.11.0-notes.md](releases/v3.11.0-notes.md)

### v3.11 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 中 | 親 dir `caglla.toml` 探索（Phase 3） | 要設計 | 未 |
| 中 | Travel Book v4 concept design | **v4.0.0 完了** | v4.1+ 実装候補 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |
| 低 | user-global config / profile | defer | 未 |

---

## v4 — Travel Book（**v4.0.0 — documentation-only**）

### テーマ

```text
旅のしおり（旅行前の共有資料）
trip export-md = Travel Book Generator v0
```

### リリース内容（v4.0.0）

```text
Travel Book の目的・情報境界・出力構成の Concept Design
Planned 中心 / Receipt 非表示 / Travel Journal 分離
Markdown 優先、PDF は defer
コード / DB / export / CLI 変更なし
```

Concept design: [v4.0.0-travel-book-concept-design.md](specifications/v4.0.0-travel-book-concept-design.md)

リリースノート: [v4.0.0-notes.md](releases/v4.0.0-notes.md)

### 到達イメージ（実装 milestone）

```text
Rich Markdown Export（export-md 拡張）
Summary / Reservation のしおり向け統合表示
PDF Export（構造安定後）
```

現行 `trip export-md` は **Travel Book Generator v0**（[summary-post-implementation-review.md](specifications/summary-post-implementation-review.md)）。製品 v4 では出力パイプラインと共有体験を強化する。

> **注:** **Reservation** は v1.18.0 で CRUD + export 済み。v4 では新規 Entity ではなく **しおりへの統合表示** が主な伸びしろ。

### v4.0 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | Travel Book Markdown structure design | **v4.1.0 完了** | — |
| 高 | Okinawa Travel Book sample enrichment | **v4.1.1 完了** | v4.1.2 |
| 中 | export-md layout improvement | v4.1.2 後 | v4.2.0 |
| 中 | Reservation / Summary display refinement | v4.2 後 | v4.3.0 候補 |
| 中 | 親 dir `caglla.toml` 探索（Phase 3） | 要設計 | 未 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |
| 低 | PDF feasibility study | v4.2+ 後 | defer |

---

## v4.1 — Travel Book chapter structure（**v4.1.0 — documentation-only**）

### テーマ

```text
Travel Book Markdown の章立て正本
章順・entity 割当・空章ルール・奥付（Colophon）
```

### リリース内容（v4.1.0）

```text
9 章: Cover → … → Notes → Colophon
常時出力: Cover / Daily schedule / Colophon
奥付に DB path は通常非表示
export-md とのギャップ整理（実装は v4.2）
コード / DB / export / CLI 変更なし
```

Chapter structure design: [v4.1.0-travel-book-chapter-structure-design.md](specifications/v4.1.0-travel-book-chapter-structure-design.md)

リリースノート: [v4.1.0-notes.md](releases/v4.1.0-notes.md)

### v4.1 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | export-md layout improvement | v4.1.2 sample 完了 | **v4.2.0** |
| 高 | Okinawa Travel Book sample enrichment | **v4.1.2 完了** | — |
| 中 | Reservation / Summary display refinement | v4.2 後 | v4.3.0 候補 |
| 中 | 親 dir `caglla.toml` 探索（Phase 3） | 要設計 | 未 |
| 中 | doctor / advisor Estimate・Receipt 活用 | 可 | 未 |
| 低 | PDF feasibility study | defer | 未 |
| 低 | user-global config / profile | defer | 未 |

---

## v4.1.1 — Okinawa Travel Book sample enrichment plan（**v4.1.1 — documentation-only**）

### テーマ

```text
Okinawa canonical sample を Travel Book fixture として拡充する計画
台帳不変（58 / 49 / ¥561,780）+ Summary / Note / Reservation / Estimate
```

### リリース内容（v4.1.1）

```text
追加対象・責務境界・itinerary 対応表（Day + sort_order + title）
golden 更新方針（trip.summary を normalize に追加）
v4.1.2 seed / v4.2.0 export-md の前段
コード / seed / golden 変更なし
```

Enrichment plan: [v4.1.1-okinawa-travel-book-sample-enrichment-plan.md](specifications/v4.1.1-okinawa-travel-book-sample-enrichment-plan.md)

リリースノート: [v4.1.1-notes.md](releases/v4.1.1-notes.md)

### v4.1.1 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | Okinawa sample enrichment | **v4.1.2 完了** | — |
| 高 | export-md layout improvement | v4.1.2 後 | **v4.2.0** |
| 中 | Reservation / Summary display refinement | v4.2 後 | v4.3.0 候補 |
| 低 | PDF feasibility study | defer | 未 |

---

## v4.1.2 — Okinawa Travel Book sample enrichment（**v4.1.2**）

### テーマ

```text
Okinawa canonical sample v1 — Travel Book fixture 実装
Summary / Note / Reservation / Estimate 追加、台帳不変
```

### リリース内容（v4.1.2）

```text
seed.sh v1 + regenerate-golden.sh
expected-export-v3.json 更新（trip.summary / reservations / estimates / notes）
58 itinerary / 49 expense / ¥561,780 / Receipt 6 維持
export-md レイアウトは v4.2.0
```

Implementation plan: [v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md](specifications/v4.1.2-okinawa-travel-book-sample-enrichment-implementation-plan.md)

リリースノート: [v4.1.2-notes.md](releases/v4.1.2-notes.md)

### v4.1.2 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | export-md layout improvement | v4.1.0 + v4.1.2 完了 | **v4.2.0 完了** |
| 中 | export-md post-release review | v4.2.0 後 | **v4.2.1**（documentation-only） |
| 中 | Travel Book Markdown polish | v4.2.1 review | **v4.2.2** 候補 |
| 中 | Reservation / Summary display refinement | v4.2 後 | v4.3.0 候補 |
| 低 | PDF feasibility study | defer | 未 |

---

## v4.2.0 — Travel Book export-md layout（**v4.2.0**）

### テーマ

```text
trip export-md を v4.1.0 章立てで実装
Okinawa sample で Travel Book として検証
```

### リリース内容（v4.2.0）

```text
Cover → Trip overview → Daily schedule → Reservations → Checklist
     → Planned cost → Notes → Colophon
Expense / Receipt / Difference は Travel Book に含めない
expected-export-md.md golden + integration test
```

Implementation plan: [v4.2.0-export-md-layout-improvement-implementation-plan.md](specifications/v4.2.0-export-md-layout-improvement-implementation-plan.md)

リリースノート: [v4.2.0-notes.md](releases/v4.2.0-notes.md)

### v4.2.0 後の着手候補

| 優先 | 候補 | 設計 | 実装 |
|---|---|---|---|
| 高 | export-md post-release review | v4.2.0 完了 | **v4.2.1** |
| 高 | Travel Book Markdown polish | v4.2.1 review | **v4.2.2** |
| 中 | Reservation / Summary display refinement | v4.2 後 | v4.3.0 候補 |
| 低 | PDF feasibility study | defer | 未 |

---

## v4.2.1 — Travel Book export-md post-release review（**documentation-only**）

### テーマ

```text
v4.2.0 出力の責務境界・可読性・文言のレビュー
沖縄 expected-export-md.md を正本
分類: Keep / Defer / Do not do → v4.2.2 へ polish を分離
```

Post-release review: [v4.2.1-travel-book-export-md-post-release-review.md](specifications/v4.2.1-travel-book-export-md-post-release-review.md)

リリースノート: [v4.2.1-notes.md](releases/v4.2.1-notes.md)

コード / golden / CLI 変更なし。

---

## v4.2.2 — Travel Book Markdown polish

### テーマ

```text
v4.2.1 review の Defer 項目を小規模実装
章立ては v4.2.0 維持
```

Implementation plan: [v4.2.2-travel-book-markdown-polish-implementation-plan.md](specifications/v4.2.2-travel-book-markdown-polish-implementation-plan.md)

リリースノート: [v4.2.2-notes.md](releases/v4.2.2-notes.md)

1. Trip overview — 全ゼロの Stay / Travel / Total 行を省略
2. Okinawa seed — ユーザー向け Remark / Estimate 文言（fixture 由来は README 正本）
3. Notes — Trip → Day → Itinerary の出力順
4. Reservations — 冗長な `Provider:` 行を省略

非対象: Venue / map provider / 移動時間自動算出 / Expense を Travel Book に追加

---

## v4.3.0 — Reservation / Summary display refinement

### テーマ

```text
Travel Book 内の Reservation / Summary 表示を実旅行向けに整える
データモデル変更なし — export-md 表示ルールのみ
```

Design: [v4.3.0-reservation-summary-display-refinement-design.md](specifications/v4.3.0-reservation-summary-display-refinement-design.md)

Implementation plan: [v4.3.0-reservation-summary-display-refinement-implementation-plan.md](specifications/v4.3.0-reservation-summary-display-refinement-implementation-plan.md)

リリースノート: [v4.3.0-notes.md](releases/v4.3.0-notes.md)

1. Summary（Trip / Day）— 配置 Keep
2. Reservations — 種別 `###` 廃止、旅程順フラット
3. 見出し provider 冗長抑制、日本語フィールドラベル、Period 人間可読化

非対象: Daily schedule 内予約ハイライト（v4.3.2+）、Venue / Maps、Expense / Receipt

Post-release review: [v4.3.1-reservation-summary-display-post-release-review.md](specifications/v4.3.1-reservation-summary-display-post-release-review.md)

---

## v4.3.1 — Reservation / Summary display post-release review（**documentation-only**）

### テーマ

```text
v4.3.0 出力の Days overview / Reservations / 責務境界のレビュー
沖縄 expected-export-md.md を正本
分類: Keep / Polish candidate / Defer / Do not do → v4.3.2 へ polish を分離
```

Post-release review: [v4.3.1-reservation-summary-display-post-release-review.md](specifications/v4.3.1-reservation-summary-display-post-release-review.md)

リリースノート: [v4.3.1-notes.md](releases/v4.3.1-notes.md)

コード / golden / CLI 変更なし。

---

## v4.3.2 — Travel Book planned cost polish

### テーマ

```text
Planned cost 表 — Note 列がすべて空のとき列省略
v4.3.1 review V9
```

Implementation plan: [v4.3.2-travel-book-planned-cost-polish-implementation-plan.md](specifications/v4.3.2-travel-book-planned-cost-polish-implementation-plan.md)

---

## v4.4.0 — Travel Book presentation model review（**documentation-only**）

### テーマ

```text
renderer 非依存の Travel Book presentation model / view model 設計レビュー
Markdown は検証用 renderer の一つ — 正本ではない
macOS native app / caglla.travel v2 前の機能検証
```

Design: [v4.4.0-travel-book-presentation-model-review.md](specifications/v4.4.0-travel-book-presentation-model-review.md)

リリースノート: [v4.4.0-notes.md](releases/v4.4.0-notes.md)

1. Domain / Presentation / Markdown renderer の三層分離
2. TravelBookItineraryItem（時刻・タイトル・カテゴリ表示名・cue 等）
3. `markdown.rs` から presentation 層へ移す候補の分類

非対象: 大規模抽象化一括導入、GUI コード、Markdown-only 業務ルール追加、DB schema 変更

---

## v4.4.1 — Category display name in Travel Book

### テーマ

```text
Daily schedule カテゴリ行を ItineraryCategory::definition().display_name に寄せる
presentation model 向けの小さな抽出 — Markdown 専用マッピングではない
```

Implementation plan: [v4.4.1-category-display-name-in-travel-book-implementation-plan.md](specifications/v4.4.1-category-display-name-in-travel-book-implementation-plan.md)

リリースノート: [v4.4.1-notes.md](releases/v4.4.1-notes.md)

例: `- Category: transport` → `- 種別: 移動`

---

## v5 — Travel Journal

### テーマ

```text
旅行記録（旅行後の振り返り）
```

### 到達イメージ

```text
Photo
Attachment（Evidence 共通レイヤー経由 — Receipt 専用 image_path は採用しない）
```

**ブロッカー:** Evidence / Attachment 設計が先。v3.8.0 時点では **設計・実装とも着手不可**。

---

## v6 — Identity

### テーマ

```text
利用者
```

### 到達イメージ

```text
User
Authentication
```

**クラウド同期はまだ行わない**。まず「利用者」という概念を導入する。

---

## v7 — Cloud

### テーマ

```text
共有と同期
```

### 到達イメージ

```text
Cloud Sync
Backup
Sharing
```

---

## v8 — Platform

### テーマ

```text
Caglla Engine
```

### 到達イメージ

```text
Desktop Application
Mobile Application
Backend Library
Public API
```

---

## 退役: 旧ロードマップ「v4 — Reservation」

以下は **製品メジャー v4 としては採用しない**（v1.18.0 で到達済み）:

```text
Reservation Entity（Itinerary 配下）
Reservation Export / Import
Reservation Display（Trip / Itinerary 一覧）
```

設計系列は v1.11–v1.19 および [v1.18.0-notes.md](releases/v1.18.0-notes.md) を参照。将来の予約 **機能拡張** は Travel Book（v4）またはパッチ設計で扱う。

---

## 一覧（早見表）

| メジャー | テーマ | ユーザーができること（イメージ） |
|---|---|---|
| **v1** | Planning Foundation | 計画・実績の記録、しおりの土台 |
| **v2** | Participant Foundation | 同行者を Trip に登録 |
| **v3** | Shared Expense | 誰が払い・誰の費用かを整理・精算 |
| **v3.1** | Estimate / Planned Budget | Itinerary 配下の予定費用・Planned total |
| **v3.2** | Database Status | 参照中 `caglla.db` の明示・状態確認 |
| **v3.2.1** | Module Layout | `src/` 責務別 module 整理（refactor） |
| **v3.3** | Planned vs Actual Difference | Trip 単位の通貨別差分（stats / export-md） — **v3.3.0 リリース済み** |
| **v3.4** | Itinerary Planned vs Actual | Itinerary 単位差分（export-md） — **v3.4.0 リリース済み** |
| **v3.5** | Receipt Inbox | concept design — **v3.5.0 リリース済み**（documentation-only） |
| **v3.6** | Receipt Inbox metadata-only | Trip-level Receipt CRUD + export v7 — **v3.6.0 リリース済み** |
| **v3.7** | Receipt assign / trash | assign / trash / restore + pending sum + export v8 — **v3.7.0 リリース済み**；v3.7.1 patch **リリース済み** |
| **v3.8** | Roadmap realignment | v4+ 再整列・次候補の開始可否 — **v3.8.0 documentation-only** |
| **v3.9** | Config and DB path foundation | `--db` / `CAGLLA_DB` / `caglla.toml` — **v3.9.0 Phase 1**；v3.9.1–v3.9.2 patches |
| **v3.10** | DB Use concept design | `db use` 永続 config — **v3.10.0 documentation-only** |
| **v3.11** | DB Use implementation | `db use` / `db use --clear` — **v3.11.0** |
| **v4** | Travel Book | 共有用しおり — v4.0 concept + v4.1 章立て + **v4.1.2 Okinawa sample** |
| **v5** | Travel Journal | 写真・添付付き旅行記（Evidence 設計が先） |
| **v6** | Identity | 利用者・アカウント |
| **v7** | Cloud | 同期・バックアップ・共有 |
| **v8** | Platform | デスクトップ / モバイル / API |
| *(退役)* | *旧 v4 Reservation* | *v1.18.0 で実装済み — 製品メジャー v4 ではない* |

---

## v1 系ドキュメント系列（参考）

Summary / Reservation はいずれも **3 段階** で設計を整理している。

```text
Responsibilities Review  →  Entity Design  →  Implementation Plan
```

| 概念 | 責務整理 | Entity Design | Implementation Plan | Hardening Review |
|---|---|---|---|---|
| **Summary** | v1.14.0 | v1.15.0（同梱） | v1.16.0 | v1.20.0 |
| **Reservation** | v1.11.0 | v1.12.0 | v1.13.0 | v1.19.0 |

実装着手順序は本ロードマップの **v1 優先（Summary → Reservation）** と整合させる。

**Summary 実装:** v1.17.0 で Trip / Day Summary を実装済み（[v1.17.0-notes.md](releases/v1.17.0-notes.md)）。

**Reservation 実装:** v1.18.0 で Itinerary 配下 Reservation を実装済み（[v1.18.0-notes.md](releases/v1.18.0-notes.md)）。保存は Itinerary 配下、Trip 一覧は表示・集約ビュー。

**v1 Hardening（documentation-first）:** v1.19.0 より、実装後の責務レビューをパッチリリースで記録する。コード変更は伴わない。

| 段階 | パッチ例 | 内容 |
|---|---|---|
| 設計 | v1.11–v1.13 / v1.14–v1.16 | Reservation / Summary 系列 |
| 実装 | v1.17.0 / v1.18.0 | Summary / Reservation CRUD + export |
| **Hardening** | **v1.19.0** | Reservation 実装後責務レビュー |
| **Hardening** | **v1.20.0** | **Summary 実装後責務再定義（Abstract / Journal 分離）** |
| **Hardening** | **v1.21.0** | **Note 実装後責務再定義（Annotation / Narrative 境界）** |
| **Hardening** | **v1.22.0** | **Expense 実装後責務定義（Transaction Record / Budget 分離）** |
| **Hardening** | **（文書のみ）** | **Planning Foundation 完了総括** — [planning-foundation-completion-review.md](specifications/planning-foundation-completion-review.md)（**tag なし**、v2.0.0 後 landing） |

---

## CLI 方向性（参考）

CLI は当面、**確認・検証・連携** を重視する。

```text
show / list / validate / export / import / diff / export-md / doctor / stats
```

細かな手入力・編集系コマンドは、将来 GUI が成熟した段階で **deprecated / obsolete 候補** になり得る。現行バージョンでは Summary 編集を含む CRUD は引き続きサポートする。

---

## 改訂

本メモはプロダクト判断に応じて更新する。実装・リリースの正は各 [releases/](releases/) ノートおよび [specifications/](specifications/) を参照する。
