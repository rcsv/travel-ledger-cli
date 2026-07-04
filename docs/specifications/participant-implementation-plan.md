# Participant Implementation Plan

Caglla.Travel CLI v2.0.0 で **Participant** を実装するための計画です。

**v2.0.0 設計フェーズ 3/6: Implementation Plan のみ。** 本書は DB migration・CLI・export schema・テストコードの変更を伴わない。実装は Issue #10 以降。

| ドキュメント | 役割 |
|---|---|
| [participant-model.md](participant-model.md) (#7) | 責務・境界・count 意味論 |
| [participant-entity-design.md](participant-entity-design.md) (#8) | テーブル・フィールド・検証骨格 |
| **本書** (#9) | 実装計画（If we build it, how） |
| [participant-post-implementation-review.md](participant-post-implementation-review.md) (#11) | 実装後レビュー（#10 完了後） |
| [export-schema.md](export-schema.md) | export JSON 構造（#10 で v4 追記） |

関連: [github-workflow.md](../github-workflow.md) / [data-model.md](../data-model.md) / [command-reference.md](../command-reference.md) / [export-import.md](../export-import.md) / [development.md](../development.md)

設計系列（Epic #6）:

```text
#7  Responsibilities Review        → participant-model.md
#8  Entity Design                  → participant-entity-design.md
#8+ Person / Trip 境界補正 (#21)
#8+ count semantics / is_self (#22)
#9  Implementation Plan             → participant-implementation-plan.md（本書）
#10 Implementation                 → CRUD + export v4（予定）
#11 Post-Implementation Review
#12 Release v2.0.0
```

---

## Purpose

#7 / #8 および PR #21 / #22 で固めた設計を、Issue #10 の実装で **迷わない工事手順書** に落とし込む。

```text
v2.0.0 で Trip-scoped participation record（is_self 付き）を
migration → domain → CLI → export v4 → tests の順で安全に導入する。
```

### 設計前提（必読）

| # | 前提 | 出典 |
|---|---|---|
| 1 | `participants` = **Trip participation record**（人そのものの正本ではない） | [participant-model.md §Conceptual model](participant-model.md#conceptual-model-person-vs-trip-participation) |
| 2 | **Participant** = その Trip の旅行参加者 **全員**（**自分を含む**）。**Companion** = 自分以外 | [participant-model.md §Participant count semantics](participant-model.md#participant-count-semantics) |
| 3 | `companion_count = count(participants) - 1` は **`is_self = true` が同一 Trip にちょうど 1 件** のときのみ | 同上 |
| 4 | participants **未登録** は **0 人ではなく unknown / not recorded** | 同上 |
| 5 | v2.0.0 で **`is_self`** 列を含める（Deferred にしない） | [participant-entity-design.md §is_self](participant-entity-design.md#is_self) |
| 6 | Root-level **Person / Traveler Profile**・`person_id` は **v2 では実装しない** | PR #21 |

---

## Background

### v1 完了時点

- Expense `paid_by_name` は文字列のみ。Trip 内の参加者一覧がない
- export は `schema_version: 3`（Note + nested Expense + Reservation）
- `trip stats` / `trip doctor` に Participant 概念はない

### v2 で追加する価値

| 課題 | 解決 |
|---|---|
| 「誰が行くか」を構造化 | `participants` テーブル + CLI CRUD |
| 一人旅・家族旅行の人数統計 | `is_self` + count 意味論 |
| v3 精算の前提 | 安定 `participants.id`（FK は v3） |
| バックアップ | export **schema v4** + `participants[]` |

### Bubble / caglla.travel の教訓

`count(participants) - 1` を無条件に使うと、一人旅未登録で **-1** になる。本計画では **統計・doctor・JSON** すべてで unknown を明示する。

---

## Source Documents

| ドキュメント | 本計画が引き継ぐ内容 |
|---|---|
| [participant-model.md](participant-model.md) | 責務、Person 境界、count 意味論、export v4 骨格 |
| [participant-entity-design.md](participant-entity-design.md) | DDL、FK なし cascade、CLI 骨格、Validation |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) | v3 まで Expense FK なし |
| [export-schema.md](export-schema.md) | v3 現行構造、unknown field 方針 |
| [ordering-model.md](ordering-model.md) | `sort_order` + `id` tie-break |
| [reservation-implementation-plan.md](reservation-implementation-plan.md) | 実装計画ドキュメントの体裁先例 |

---

## Implementation Scope

### v2.0.0 で行う（Issue #10）

| 領域 | 内容 |
|---|---|
| **DB** | `participants` テーブル migration（`is_self` 含む） |
| **Domain** | `Participant` struct、`src/participant.rs` |
| **Repository** | create / list / get / update / delete / cascade / duplicate |
| **Validation** | `name` 必須・trim 後非空、非負 `sort_order`、**`is_self` 最大 1 件** |
| **CLI** | `participant add` / `list` / `show` / `update` / `delete` |
| **Trip 連携** | `trip delete` manual cascade、`trip duplicate` で Participant 複製 |
| **Export** | `schema_version: 4`、top-level `participants[]`、`is_self` 出力 |
| **Import** | v4 participants 復元、v3 は `participants` 省略 = 空 |
| **validate-export** | v4 ルール（multiple self 拒否等） |
| **diff** | Participant 差分（added / removed / renamed / reordered / is_self changed） |
| **export-md** | Trip Overview に参加者一覧（§Markdown Export Plan） |
| **stats** | `participant_count` / `companion_count` / `self_known`（§Trip Stats Plan） |
| **doctor / advisor** | self / participants 未登録の **info** warning（§Doctor Plan） |
| **docs** | command-reference、export-schema、export-import、samples（任意） |
| **tests** | unit + CLI integration + export roundtrip |

### v2.0.0 で行わない（Non-goals）

| 項目 | 送り先 |
|---|---|
| Person / Traveler Profile、`persons` テーブル | 将来 Root |
| `person_id`、`trip_participants` rename | 将来 migration |
| Expense `paid_by_participant_id`、shares、Settlement | v3 Shared Expense |
| Reservation `guest_participant_id` | 将来 |
| User account、permission、cloud sync | v7+ |
| `memo` / `role` 列 | Deferred（Entity Design） |
| Travel Book 最適化 | v5 |
| Journal / Photo 紐づけ | v6 |
| `paid_by_name` 自動解決 | v3 |
| config / profile からの self 自動作成 | 将来 |

---

## Table / Migration Plan

### テーブル名

```text
participants
```

リネームは v2 では行わない（将来 `trip_participants` は Open Question）。

### DDL（v2.0.0）

```sql
CREATE TABLE IF NOT EXISTS participants (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id     INTEGER NOT NULL,
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    is_self     INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_participants_trip
    ON participants(trip_id);

CREATE UNIQUE INDEX IF NOT EXISTS idx_participants_one_self_per_trip
    ON participants(trip_id) WHERE is_self = 1;
```

### 論点整理

| 論点 | 方針 |
|---|---|
| **primary key** | `id` AUTOINCREMENT（既存 entity と同型） |
| **`trip_id` index** | **必要** — `list --trip`、cascade、duplicate |
| **`sort_order` index** | **v2 では不要** — Trip 内件数は少ない想定。`WHERE trip_id = ? ORDER BY sort_order, id` で十分 |
| **`is_self` index** | **partial unique index** で制約 + 検索を兼ねる |
| **FK** | **張らない** — Note / Expense / Reservation と同型（[participant-entity-design.md §外部キー](participant-entity-design.md#外部キー方針)） |
| **manual cascade** | `trip delete` → `delete_participants_for_trip`（`src/trip.rs` 既存 delete チェーンに追加） |
| **`is_self` 最大 1 件** | **partial unique index + アプリ側検証**（併用）。CLI は人間向けエラーメッセージを返す |
| **既存 DB への影響** | 新規テーブル追加のみ。既存 Trip は Participant 0 件のまま |
| **冪等性** | `CREATE TABLE IF NOT EXISTS` / `CREATE INDEX IF NOT EXISTS`。`migrate_participants` を 2 回実行しても安全 |
| **rollback** | **しない** — v1 migration 慣習に合わせ down migration は作らない |

### Migration 実装案

| 項目 | 案 |
|---|---|
| **関数名** | `migrate_participants(conn: &Connection) -> Result<()>` |
| **配置** | `src/participant.rs` 内、または `src/db.rs` から呼び出し |
| **呼び出し** | `init_db` / `open_db` パスで `migrate_participants` を既存 migrate 列の末尾に追加 |
| **テスト** | `test_init_db_creates_participants_table`（`db.rs` tests と同型） |

### `is_self` 制約の二重化

```text
DB 層:  partial UNIQUE INDEX (trip_id) WHERE is_self = 1
        → 競合 INSERT/UPDATE は SQLITE_CONSTRAINT になる

App 層: add --self / update --self 前に count チェック
        → 「only one self participant per trip」等の CLI メッセージ
```

アプリ層のみだと race で抜けうるため、**index を正** とし、アプリ層は UX 用。

---

## Domain / Repository Plan

### Model（`src/models.rs`）

```rust
// 案 — #10 で確定
pub struct Participant {
    pub id: i64,
    pub trip_id: i64,
    pub name: String,
    pub sort_order: i32,
    pub is_self: bool,
    pub created_at: String,
    pub updated_at: String,
}
```

Export 用:

```rust
pub struct ExportParticipantV4 {
    pub name: String,
    pub sort_order: i32,
    pub is_self: bool,
}
```

内部 ID は export **しない**（Entity Design 踏襲）。

### Module（`src/participant.rs`）

Expense / Note と同型の **専用モジュール**。`main.rs` に `mod participant;` を追加。

### Repository 操作

| 関数 | 用途 |
|---|---|
| `create_participant(conn, trip_id, name, sort_order, is_self) -> Result<i64>` | add |
| `list_participants_by_trip(conn, trip_id) -> Result<Vec<Participant>>` | list。`ORDER BY sort_order ASC, id ASC` |
| `get_participant(conn, id) -> Result<Participant>` | show |
| `update_participant(conn, id, name?, sort_order?, is_self?) -> Result<()>` | update |
| `delete_participant(conn, id) -> Result<()>` | delete |
| `delete_participants_for_trip(conn, trip_id) -> Result<()>` | trip delete cascade |
| `duplicate_participants_for_trip(conn, src_trip_id, dst_trip_id) -> Result<()>` | trip duplicate |
| `count_self_participants_for_trip(conn, trip_id) -> Result<i64>` | 制約検証 |
| `clear_self_for_trip(conn, trip_id) -> Result<()>` | `update --self` 時の既存 self 解除 |
| `compute_participant_counts(conn, trip_id) -> Result<ParticipantCounts>` | stats / JSON list |

`set_self_participant` / `clear_self_participant` は上記の合成でよい（専用 1 関数にしなくても可）。

### `ParticipantCounts`（集計ヘルパ）

```rust
pub struct ParticipantCounts {
    pub registered_count: usize,      // len(participants)
    pub participant_count: Option<usize>, // self 含む旅行人数（算出可能時のみ）
    pub companion_count: Option<usize>,   // 自分以外（算出可能時のみ）
    pub self_known: bool,                 // is_self=true がちょうど 1 件
    pub participants_recorded: bool,      // registered_count > 0
}
```

算出規則（[participant-model.md §推奨する算出規則](participant-model.md#participant-count-semantics) 踏襲）:

| 状態 | `participant_count` | `companion_count` | `self_known` |
|---|---|---|---|
| 0 件 | `None` | `None` | `false` |
| N 件、`is_self` 1 件 | `Some(N)` | `Some(N - 1)` | `true` |
| N 件、`is_self` 0 件 | `None` | `None` | `false` |
| `is_self` 2 件以上 | **データ不正** — doctor で検出、通常は DB 制約で防止 |

### 定数

```rust
pub const MAX_PARTICIPANT_NAME_LEN: usize = 200;
```

---

## CLI Plan

既存パターン（`note`、`expense`、`reservation`）に合わせる。

### コマンド一覧

```bash
participant add    --trip <trip_id> --name <name> [--sort-order N] [--self]
participant list   --trip <trip_id> [--json]
participant show   <participant_id> [--json]
participant update <participant_id> [--name <name>] [--sort-order N] [--self] [--not-self]
participant delete <participant_id>
```

### `participant add`

| 項目 | 方針 |
|---|---|
| **`--trip`** | **必須**。存在しない trip_id はエラー |
| **`--name`** | **必須**。trim 後空禁止。最大 200 文字 |
| **`--sort-order`** | 任意。省略時は **既存 max(sort_order) + 1**（0 件なら `0`）— Checklist 生成と同型 |
| **`--self`** | 指定時 `is_self = true`。同一 Trip に既に self がある場合は **エラー**（Entity Design 踏襲） |
| **self 競合時** | メッセージ例: `trip already has a self participant; use participant update --self to change` |
| **`--replace-self`** | **v2 では導入しない** — add ではエラー、update で移す |
| **human output** | `Added participant 3: Alex (trip 1)`。`--self` 時は `(self)` を付記 |
| **`--json`** | 作成された Participant オブジェクト 1 件 |

### `participant update`

| 項目 | 方針 |
|---|---|
| **対象** | `participant_id`（DB 全体で一意） |
| **`--name` / `--sort-order`** | 任意。少なくとも 1 つ必須（既存 update コマンドと同型） |
| **`--self`** | この行を self に。**同一 Trip の他行 `is_self` を先に false にしてから** true にする（トランザクション内） |
| **`--not-self`** | この行の `is_self` を false に。**self が 0 件になることを許す** |
| **self 0 件** | 許容 — stats は `self_known = false`、companion_count は unknown |

### `participant list`

| 項目 | 方針 |
|---|---|
| **ソート** | `sort_order ASC, id ASC` |
| **human** | 表形式: `ID  NAME  SORT  SELF`。self は `*` または `yes` |
| **counts（human）** | self_known 時のみ `Participants: N (companions: M)` をフッターに表示。否则 `Participants: N recorded (traveler count unknown)` |
| **`--json`** | §JSON Output Plan |

### `participant show`

| 項目 | 方針 |
|---|---|
| **human** | id, trip_id, name, sort_order, is_self, created_at, updated_at |
| **`--json`** | 単一 Participant オブジェクト |

### `participant delete`

| 項目 | 方針 |
|---|---|
| **self 削除** | **許可** — 削除後 self 0 件になりうる |
| **FK** | v2 では他 entity 参照なし → 単純 DELETE |
| **v3 注意** | Deferred note: Expense 参照時は RESTRICT 等を検討 |

### エラーメッセージ（案）

| 条件 | メッセージ |
|---|---|
| name 空 | `name must not be empty` |
| sort_order 負 | `sort_order must be non-negative` |
| trip 不在 | `trip not found` |
| participant 不在 | `participant not found` |
| add --self 競合 | `trip already has a self participant` |
| update 対象なし | `at least one of --name, --sort-order, --self, --not-self is required` |

---

## Default Self Participant Policy

### 検討した案

| 案 | 内容 | 評価 |
|---|---|---|
| **A** | `trip add --self-name` で self participant 同時作成 | 将来拡張として **#10 Open**。v2 必須ではない |
| **B** | 作成後に `participant add --self` を明示 | **推奨フロー**（Getting Started に記載） |
| **C** | config / default profile から自動 | User なしの local-first では **v2 対象外** |
| **D** | 自動作成しない。stats は unknown 許容 | **v2 のデフォルト方針** |

### 採用方針（v2.0.0）

```text
D を基本とし、B をドキュメントで推奨する。
trip add では Participant を自動作成しない。
```

理由:

- local-first CLI に User account がない — デフォルト名が不明
- 強制自動作成は既存 Trip / import との整合が複雑
- unknown 表示 + doctor warning で統計破綻を防ぐ

### 補完施策（v2.0.0 で実装）

| 施策 | 内容 |
|---|---|
| **`trip stats --json`** | participants 未登録 → `participant_count: null`, `self_known: false` |
| **`trip doctor`** | info: `PARTICIPANTS_NOT_RECORDED`（0 件）、`SELF_PARTICIPANT_UNKNOWN`（件数 > 0 かつ self 0 件） |
| **`trip advisor`** | 上記 issue に `participant add --self` 等の try ヒント（任意） |
| **Getting Started / README** | 家族旅行例で `participant add --self` を示す |
| **canonical sample** | #10 完了後の **任意タスク** — 投入時は **self 1 件を含める**（Alex = self 等） |

### 将来（A の検討）

```bash
trip add "沖縄" --start ... --end ... --self-name "Alex"
```

同一トランザクションで Trip + Day + self Participant を作る。**#10 では必須にしない**。

---

## JSON Output Plan

### `participant list --trip <id> --json`

```json
{
  "schema_version": 1,
  "trip_id": 1,
  "participants": [
    {
      "id": 1,
      "trip_id": 1,
      "name": "ともさん",
      "sort_order": 0,
      "is_self": true,
      "created_at": "2026-06-19 12:00:00",
      "updated_at": "2026-06-19 12:00:00"
    }
  ],
  "counts": {
    "registered_count": 1,
    "participant_count": 1,
    "companion_count": 0,
    "self_known": true,
    "participants_recorded": true
  }
}
```

### self 不明時（登録あり、is_self 0 件）

```json
{
  "counts": {
    "registered_count": 2,
    "participant_count": null,
    "companion_count": null,
    "self_known": false,
    "participants_recorded": true
  }
}
```

### 未登録時（0 件）

```json
{
  "participants": [],
  "counts": {
    "registered_count": 0,
    "participant_count": null,
    "companion_count": null,
    "self_known": false,
    "participants_recorded": false
  }
}
```

### `participant show <id> --json`

単一 Participant オブジェクト（`counts` は **含めない** — list 専用）。

### 既存 JSON 方針との整合

- ルートに `schema_version`（participant コマンド用は `1` から開始）
- `null` で unknown を表現（`trip stats --json` と同型）
- 内部フィールド名は snake_case

---

## Export / Import Plan

### schema version 昇格条件

**いずれかが満たされたら v4 を出力:**

```text
trip export が participants[] を含む
→ schema_version: 4
```

v2.0.0 実装後、**常に v4** を出力（Participant 0 件でも `participants: []`）。

### Top-level 構造（v4）

```json
{
  "schema_version": 4,
  "generator": "caglla-cli",
  "generator_version": "2.0.0",
  "exported_at": "...",
  "trip": { "name": "...", "start_date": "...", "end_date": "..." },
  "participants": [
    { "name": "ともさん", "sort_order": 0, "is_self": true },
    { "name": "妻", "sort_order": 1, "is_self": false }
  ],
  "days": [],
  "checklist_items": [],
  "notes": []
}
```

| 論点 | 方針 |
|---|---|
| **内部 ID** | export **しない** |
| **`is_self`** | **必須**（boolean）。省略 import は `false` 扱いにしない — **明示必須**（#10 で validate） |
| **`is_self` 0 件** | **許可** — import 後 stats は unknown |
| **multiple `is_self: true`** | **validate-export で invalid** |
| **`sort_order`** | そのまま保持。配列順のみに依存しない |
| **unknown field** | 既存方針 — 未知キーは import で無視（警告可） |
| **v3 → v4 import** | `participants` 省略 → 空配列 |
| **v4 → v3 import** | **不可**（v3 は participants 未知）— 想定どおり |
| **v3 export 互換** | v3 export の import は **継続** |

### Import 順序

```text
1. Trip 作成
2. Participants 作成（ID 再採番）
3. Days / Itineraries
4. Checklist, Notes, Reservations
5. Expenses（paid_by_name のみ — Participant 解決なし）
```

`trip import` / `import_from_export_json` の既存フローに **手順 2** を挿入。

### validate-export 追加（v4）

| チェック | 結果 |
|---|---|
| `schema_version: 4` で `participants` 欠落 | warning または invalid（#10 で決定 — **空配列必須** を推奨） |
| `name` 空 | invalid |
| `sort_order` 負 | invalid |
| 同一 Trip 内 `is_self: true` が 2 件以上 | **invalid** |
| `is_self` 型不正 | invalid |

### Roundtrip テスト

`tests/export_roundtrip_cli.rs` に v4 + participants ケースを追加（#10）。

---

## Diff Plan

### 対象

`trip diff <old.json> <new.json>` に Participant 差分を追加。

### 差分カテゴリ

| 種別 | 検出 |
|---|---|
| **added** | 新 JSON にのみ存在 |
| **removed** | 旧 JSON にのみ存在 |
| **renamed** | 同一キーで `name` 変更 |
| **reordered** | `sort_order` 変更 |
| **is_self changed** | 同一キーで `is_self` 変更 |

### Diff identity（export に internal id がない）

**複合キー:** `(sort_order, name)` を第一キーとする。

| 状況 | 扱い |
|---|---|
| 同名 Participant 許可（v2 方針） | 同一 `sort_order` + `name` が複数ある場合は配列内 **出現順序** でペアリング（Note diff と同型の限界を文書化） |
| rename + reorder 同時 | `removed` + `added` として検出（既存 diff の保守的パターン） |

### `TripDiff` 拡張（案）

```rust
pub participant_added: Vec<ExportParticipantV4>,
pub participant_removed: Vec<ExportParticipantV4>,
pub participant_changed: Vec<ParticipantFieldChange>,
```

v3 同士の diff では `participants` は **空扱い**（フィールドなし = `[]`）。

---

## Markdown Export Plan

### 方針: Trip Overview に参加者一覧を出す

| 項目 | 方針 |
|---|---|
| **配置** | Trip Overview（既存 summary の近く）に `## Participants` セクション |
| **Day / Itinerary** | **出さない** — Participant は Trip スコープ |
| **順序** | `sort_order`, `id` 順 |
| **self 表示** | 名前の後に ` (self)` または `*` |
| **companion_count** | **self_known 時のみ** `Travelers: N, Companions: M` を Overview に 1 行 |
| **self 不明** | `Travelers: unknown (2 participants recorded)` — **0 人と表示しない** |
| **0 件** | セクション **省略** または `Participants: not recorded`（#10 でどちらか選択 — **省略推奨**） |

`export-md` は表示レイヤー。データ正本は `participants` テーブル。

---

## Trip Stats / Doctor / Advisor Plan

### `trip stats`（v2.0.0 で拡張）

`TripStats` / `trip stats --json` に追加:

| フィールド | 型 | 説明 |
|---|---|---|
| `participants_recorded` | bool | `registered_count > 0` |
| `registered_participant_count` | usize | DB 行数 |
| `participant_count` | `Option<usize>` | 旅行人数（self 含む）。unknown は `null` |
| `companion_count` | `Option<usize>` | 自分以外。unknown は `null` |
| `self_known` | bool | `is_self=true` が 1 件 |

**text 出力:** 既存統計の末尾に 1–2 行追加。未登録時は `Participants: not recorded`。

### `trip doctor`（v2.0.0 で追加 — info レベル）

| Code | 条件 | Severity |
|---|---|---|
| `PARTICIPANTS_NOT_RECORDED` | participants 0 件 | info |
| `SELF_PARTICIPANT_UNKNOWN` | participants > 0 かつ self 0 件 | info |
| `MULTIPLE_SELF_PARTICIPANTS` | self 2 件以上（DB 破損検出） | error |

既存 itinerary 系 warning より **優先度は低い**（旅行計画の補助情報）。

### `trip advisor`（v2.0.0 — 任意だが推奨）

上記 issue に対し:

```text
Try: participant add --trip <id> --name "Your name" --self
```

`--with-commands` 時のみ（既存 advisor パターン）。

---

## Testing Plan

### Unit tests（`src/participant.rs`）

| テスト | 内容 |
|---|---|
| migration | テーブル・index 作成、冪等性 |
| create / list / get / update / delete | 基本 CRUD |
| name validation | 空文字拒否、trim、最大長 |
| sort_order | 負数拒否、tie-break 順 |
| `--self` 相当 | create with is_self |
| self 最大 1 件 | 2 件目はエラー |
| self 0 件 | 許容、counts が unknown |
| `compute_participant_counts` | 全パターン（0 件 / self 1 / self 0 / 一人旅） |
| cascade | `delete_participants_for_trip` |
| duplicate | `duplicate_participants_for_trip` — name / sort_order / is_self 複製、新 ID |

### CLI integration tests（`tests/participant_cli.rs` 新規）

| テスト | 内容 |
|---|---|
| add / list / show | human + `--json` |
| add --self | self マーク |
| self 競合 | add --self 2 回目エラー |
| update --self | 既存 self の付け替え |
| update --not-self | self 解除 |
| delete | 通常 + self 削除 |
| not found | 既存 `not_found_cli` パターン |

### Export / import

| テスト | 内容 |
|---|---|
| export v4 | `schema_version: 4`, `participants[]` |
| import v4 | roundtrip |
| import v3 | participants なし → 空 |
| invalid export | multiple self → validate-export fail |
| reexport | structural roundtrip（既存 checklist / notes パターン） |

### その他

| テスト | 内容 |
|---|---|
| diff | added / removed / is_self changed |
| markdown | Overview に Participants セクション |
| trip stats | null counts |
| doctor | PARTICIPANTS_NOT_RECORDED / SELF_PARTICIPANT_UNKNOWN |
| trip delete cascade | participants 削除 |
| trip duplicate | participants 複製 |
| JSON output | counts ブロック |

### canonical sample / golden

| 項目 | 方針 |
|---|---|
| **okinawa_sesoko_2026** | Participant 投入は **#10 後の任意タスク**。投入時は self 含む |
| **golden** | export 期待値 JSON 更新が必要なら #10 で実施 |

---

## Implementation Sequence

Issue #10 向け推奨順序（依存関係順）:

```text
 1. models.rs — Participant, ExportParticipantV4, ParticipantCounts
 2. participant.rs — migrate_participants + CRUD + counts helper
 3. db.rs — init_db から migrate 呼び出し
 4. self 制約 — partial unique index + アプリ検証 + unit tests
 5. main.rs — participant サブコマンド配線
 6. participant add / list / show（human + JSON）
 7. participant update / delete
 8. trip delete — delete_participants_for_trip
 9. trip duplicate — duplicate_participants_for_trip
10. trip export — schema v4, participants[]
11. trip import — participants 復元、ID remap
12. validate-export — v4 ルール
13. diff — participant 差分
14. export-md — Trip Overview Participants セクション
15. stats — participant_count / companion_count / self_known
16. doctor / advisor — info warnings + try hints
17. docs — command-reference, export-schema, export-import
18. CLI integration tests + export roundtrip tests
19. canonical sample（任意）
20. make check
```

**早めに export v4 を入れる理由:** import / validate / diff / roundtrip が Participant CRUD に依存するため、**6–9 の後 10–12 をまとめて** 着手するのが効率的。

---

## Compatibility / Risk Notes

| リスク | 影響 | 緩和 |
|---|---|---|
| **schema v4 昇格** | v3 import ツールが v4 を読めない | 想定内。v4 export は v2 CLI から |
| **v3 export import** | 継続必須 | participants なしコードパスを維持 |
| **is_self 制約** | 競合 UPDATE | partial unique index + トランザクション |
| **self 未登録 stats** | 0 人誤表示 | `null` / `not recorded` をテストで固定 |
| **同名 Participant** | diff / 表示の曖昧さ | sort_order + name キー、一覧で ID 表示 |
| **FK なし** | 孤児 trip_id | create 時 trip 存在チェック |
| **sort_order 重複** | 表示順の揺れ | `id` tie-break を維持 |
| **将来 Person** | migration 追加 | `person_id` nullable 列追加で足せる。`is_self` は残す |
| **将来 v3 Expense FK** | Participant 削除制約 | #10 では削除自由。v3 で RESTRICT 検討 |
| **canonical sample 更新** | golden 差分 | 任意タスクとして分離 |

---

## Deferred Scope

### v3 Shared Expense

```text
expenses.paid_by_participant_id
expense_beneficiaries
Settlement
Participant 削除制約
export: paid_by_participant_ref
paid_by_name との backfill スクリプト（任意）
```

### Future Person / Traveler Profile

```text
persons / traveler_profiles テーブル（Root）
participants.person_id nullable FK
is_self は Trip participation 側マーカーとして維持
パスポート・マイレージ等は Person 側
```

### v5 Travel Book

```text
しおり同行者セクションのリッチ表示
export-md より高度なレイアウト
```

### v6 Travel Journal

```text
Photo / Journal と Participant の紐づけ
```

---

## Open Questions

#10 実装時に最終決定:

| # | 質問 | 本書の推奨 |
|---|---|---|
| 1 | `participant add` のデフォルト `sort_order` | **max + 1** |
| 2 | export v4 に `created_at` / `updated_at` を含めるか | **含めない**（内部 ID と同様） |
| 3 | export-md で participants 0 件のときセクション省略か明示か | **省略** |
| 4 | `trip add --self-name` を v2 に入れるか | **v2 必須ではない**（将来） |
| 5 | doctor `PARTICIPANTS_NOT_RECORDED` を warning に昇格するか | **info のまま** |
| 6 | validate-export: v4 で `participants` キー必須か | **必須（空配列可）** |
| 7 | 同一 Trip 内 `name` 重複を UI でどう区別するか | list に ID 列を常に表示 |

---

## Completion Criteria

Issue #9（Implementation Plan）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `participant-implementation-plan.md` が存在する | 本書 |
| 2 | #7 / #8 / #21 / #22 と整合 | §Purpose, §Source Documents |
| 3 | 実装対象・非対象が明確 | §Implementation Scope, §Non-goals |
| 4 | migration / repository / CLI / export / diff / stats / tests が整理 | 各 § |
| 5 | `is_self` と count semantics が反映 | §Domain, §CLI, §JSON, §Stats |
| 6 | default self policy が整理 | §Default Self Participant Policy |
| 7 | v2.0.0 実装順序が明確 | §Implementation Sequence |
| 8 | v3 / Person 境界が明確 | §Deferred Scope |
| 9 | Rust / DB / export 実装なし | 本フェーズ対象外 |

---

## Next phase notes（Implementation #10）

#10 で本計画に従いコード変更を行った（PR #24）。#11 [Post-Implementation Review](participant-post-implementation-review.md) 完了後 → #12 Release v2.0.0。

優先確認:

- partial unique index が target SQLite で動作すること（CI / ローカル）
- v3 export roundtrip 回帰が壊れていないこと
- `make check` 全通過
