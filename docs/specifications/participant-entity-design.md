# Participant Entity Design

Caglla.Travel CLI / 将来 Web 版に向けた **Participant エンティティ** の具体設計です。

**v2.0.0 設計フェーズ 2/6: Entity Design のみ。** 本書は DB migration・CLI・export schema の変更を伴わない。実装手順は Issue #9 以降。

| ドキュメント | 役割 |
|---|---|
| [participant-model.md](participant-model.md) (#7) | 責務・境界（What it is / is not） |
| **本書** (#8) | テーブル・フィールド・関係・検証（How we model it） |
| [participant-implementation-plan.md](participant-implementation-plan.md) (#9 予定) | 実装計画（If we build it, how） |

関連: [note-model.md](note-model.md) / [expense-model.md](expense-model.md) / [ordering-model.md](ordering-model.md) / [export-schema.md](export-schema.md) / [github-workflow.md](../github-workflow.md)

設計系列（Epic #6）:

```text
#7  Responsibilities Review   → participant-model.md
#8  Entity Design             → participant-entity-design.md（本書）
#9  Implementation Plan        → participant-implementation-plan.md（予定）
#10 Implementation             → CRUD + export v4（予定）
```

---

## Purpose

[participant-model.md](participant-model.md) で確定した責務を前提に、v2.0.0 で導入する **Participant entity の保存形・フィールド・境界** を具体化する。

```text
後続の Implementation Plan / 実装が迷わないよう、
テーブル・フィールド・ cascade・export 骨格を固める。
```

---

## Background

### v1 での「誰」の表現

| 手段 | 限界 |
|---|---|
| `expenses.paid_by_name` | 文字列のみ。Trip 内の同行者一覧がない |
| Note / Summary 本文 | 非構造化 |

### v2 の到達点（Entity レベル）

```text
participants テーブル — Trip スコープの同行者正本
最小フィールド: name + sort_order + 監査列
```

精算・Expense FK・Reservation リンクは **v3 以降**（[participant-model.md §Deferred](participant-model.md#deferred-scope)）。

---

## Source Responsibilities Review

[participant-model.md](participant-model.md) から本設計が引き継ぐ前提:

| 項目 | 前提 |
|---|---|
| **親** | Trip のみ |
| **スコープ** | 同行者レジストリ。精算ではない |
| **Expense** | v2 では構造リンクなし |
| **Reservation** | v2 では直接リンクなし |
| **export** | 将来 schema v4、`participants[]` top-level |
| **cascade** | Trip 削除で Participant 全削除 |

本書は上記を **破らない** 範囲でフィールドを確定する。

---

## Entity Definition

### What Participant is

```text
Participant is a Trip-scoped identity for a person who travels on that trip.
```

日本語:

```text
Participant = ある Trip に同行する人を表すエンティティ（Trip 内 ID）
```

### What Participant is not（v2.0.0）

| 誤解しやすい概念 | 関係 |
|---|---|
| **User account** | ではない — Identity（製品 v7）の領域 |
| **連絡先帳 / アドレス帳** | ではない — グローバル再利用しない |
| **Expense payer / debtor** | **まだではない** — v3 で Expense と構造リンク |
| **Reservation guest** | **まだではない** — v2 では Reservation に Participant 列なし |
| **Settlement 単位** | ではない — v3 |
| **Checklist 担当者** | **まだではない** — 将来 optional |

### エンティティ関係（v2.0.0）

```text
Trip
 ├─ Participant[]     ← 本設計
 ├─ Checklist[]
 └─ Day
      └─ Itinerary
           ├─ Expense      （paid_by_name のみ — Participant FK なし）
           ├─ Reservation  （Participant FK なし）
           └─ Note
```

---

## Table Design

### テーブル名

```text
participants
```

### DDL 案（v2.0.0）

```sql
CREATE TABLE participants (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    trip_id     INTEGER NOT NULL,
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_participants_trip
    ON participants(trip_id);
```

### 外部キー方針

Note / Expense / Reservation と同型の **案 C: FK なし + アプリ側 cascade**（[note-model.md §4](note-model.md#4-外部キー--cascade-方針)）。

| 理由 | 説明 |
|---|---|
| 既存慣習 | `notes` / `expenses` と統一 |
| 削除経路 | `trip delete` から `delete_participants_for_trip` を明示呼び出し |
| 検証 | create 時に `trip_id` 存在チェック |

`trip_id` は `trips.id` を指すが、SQLite FK 制約は **張らない**（実装時）。

### v2.0.0 に含めない列（Deferred）

| 列候補 | 判定 | 送り先 |
|---|---|---|
| `memo` | **v2 では不要** | 自由文は Note。必要なら将来列または Note 併用 |
| `role` | **v2 では不要** | 親/子/同行者等 — v3 以降または Travel Book |
| `paid_by_participant_id`（Expense 側） | **v2 では不要** | v3 |
| `user_id` | **不要** | v7 Identity |

---

## Fields

### id

| 項目 | 方針 |
|---|---|
| **目的** | 行の一意識別。CLI `show` / `update` / `delete` のキー |
| **必須** | はい（DB 自動採番） |
| **型** | `INTEGER PRIMARY KEY AUTOINCREMENT` |
| **制約** | — |
| **CLI 表示** | `participant list` の ID 列 |
| **export** | **export しない**（内部 ID — Expense / Reservation と同型） |
| **将来** | v3 で `paid_by_participant_id` の参照先 |

### trip_id

| 項目 | 方針 |
|---|---|
| **目的** | 親 Trip。Participant のスコープ境界 |
| **必須** | はい |
| **型** | `INTEGER NOT NULL` |
| **制約** | 存在する `trips.id` であること（アプリ検証） |
| **CLI 表示** | `list --trip` では冗長のため JSON のみ、または省略表示 |
| **export** | 暗黙（`participants[]` は Trip export の文脈）— 明示 `trip_id` は **不要** |
| **将来** | 変更なし |

### name

| 項目 | 方針 |
|---|---|
| **目的** | Trip 内での **表示名**（しおり・一覧・将来精算ラベル） |
| **必須** | はい |
| **型** | `TEXT NOT NULL` |
| **制約** | trim 後 **空文字禁止**。最大長は Implementation で定数化（案: 200 文字） |
| **CLI 表示** | `participant list` の主列。`add --name` / `update --name` |
| **export** | `name` フィールドとして出力 |
| **将来** | `display_name` へのリネームは **不要**（`name` で統一） |

**`paid_by_name` との関係:** v2 では自動連携しない。ユーザーが同じ文字列を Expense に書く運用は許容（[participant-model.md](participant-model.md)）。

### sort_order

| 項目 | 方針 |
|---|---|
| **目的** | 同一 Trip 内の表示順 |
| **必須** | はい（デフォルト `0`） |
| **型** | `INTEGER NOT NULL DEFAULT 0` |
| **制約** | 負数は **拒否**（CLI 検証） |
| **CLI 表示** | 一覧は `sort_order → id` でソート |
| **export** | `sort_order` を出力 |
| **将来** | 変更なし |

### created_at / updated_at

| 項目 | 方針 |
|---|---|
| **目的** | 監査・デバッグ。既存エンティティと同形式 |
| **必須** | はい |
| **型** | `TEXT NOT NULL` — `YYYY-MM-DD HH:MM:SS`（`db::now_string()`） |
| **制約** | — |
| **CLI 表示** | `show` / `--json` で任意表示 |
| **export** | v2 export v4 では **省略可**（Implementation Plan で最終判断） |
| **将来** | 必要なら export に追加 |

---

## Relationships

### Trip

| 関係 | v2.0.0 |
|---|---|
| **カーディナリティ** | 1 Trip : 0..N Participant |
| **正本の親** | Trip |
| **削除** | Trip 削除 → 配下 Participant **全削除** |
| **複製** | `trip duplicate` → Participant も複製（新 `trip_id`、新 `id`） |

### Day

**直接関係なし。** Day は日付コンテナ。Participant は Trip 全体に同行する。

### Itinerary

**直接関係なし（v2）。** 行動の正本は Itinerary。Participant は「誰が Trip にいるか」の集合。

### Expense

| 関係 | v2.0.0 |
|---|---|
| **DB リンク** | **なし** |
| **`paid_by_name`** | 維持。文字列記録 |
| **v3** | `expenses.paid_by_participant_id` → `participants.id`（Deferred） |

```text
v2: Expense と Participant は論理的に独立。
v3: Expense が Participant を参照し始める。
```

### Reservation

| 関係 | v2.0.0 |
|---|---|
| **DB リンク** | **なし** |
| **方針** | [reservation-responsibilities-review.md](reservation-responsibilities-review.md) 維持 |

予約名義・ゲスト情報は Reservation / Note の領域。

### Note

| 関係 | v2.0.0 |
|---|---|
| **エンティティ** | 別物 |
| **本文** | 「長男向けの注意」等は Note に記載可 — Participant 正本ではない |

### Summary

| 関係 | v2.0.0 |
|---|---|
| **責務** | Abstract — 参加者リストの正本ではない |
| **将来** | Generator が Participant 数を参照しうる |

### Checklist

| 関係 | v2.0.0 |
|---|---|
| **スコープ** | ともに Trip 配下 |
| **担当割当** | **なし** — `assigned_participant_id` は Deferred |

---

## Ordering Policy

[ordering-model.md](ordering-model.md) および Checklist / Note / Expense と同型:

```text
同一 Trip 内の表示順:
  ORDER BY sort_order ASC, id ASC
```

| 論点 | 方針 |
|---|---|
| **Sequence-first** | Itinerary の sequence とは **独立** — Participant 独自の `sort_order` |
| **追加時** | `sort_order` 未指定 → `0`。既存最大+1 は Implementation Plan で選択可 |
| **重複** | 同一 `sort_order` は **許可** — `id` で安定 tie-break |
| **reorder** | v2.0.0 では専用コマンド **なし**。`update --sort-order` で十分（#9 で確定） |

---

## Deletion / Cascade Policy

### Trip 削除

```text
trip delete
  → delete_participants_for_trip(trip_id)
  → （既存）itineraries, notes, expenses, reservations, checklist…
```

トランザクション内で実行（Note cascade と同型）。

### Participant 単体削除

```text
participant delete <id>
  → DELETE FROM participants WHERE id = ?
```

v2.0.0 では **他テーブルからの FK 参照がない** ため、単純削除でよい。

### Day / Itinerary 削除

Participant には **影響なし**（Trip スコープのため）。

### 将来（v3）— Deferred

| 状況 | 検討事項 |
|---|---|
| Expense が `paid_by_participant_id` を参照 | Participant 削除時 — RESTRICT / SET NULL / 禁止 |
| Settlement 存在 | 削除ポリシーは v3 設計で決定 |

本書では **Open Questions** に記録のみ。

---

## CLI Design Outline

**本フェーズでは実装しない。** v2.0.0 で必要な操作:

| コマンド | 目的 | 必須オプション（案） |
|---|---|---|
| `participant add` | 同行者追加 | `--trip`, `--name` |
| `participant list` | Trip 内一覧 | `--trip` |
| `participant show` | 1 件表示 | `<participant_id>` |
| `participant update` | 名前・順序変更 | `<participant_id>`, `--name` / `--sort-order` |
| `participant delete` | 削除 | `<participant_id>` |

### 方針

| 論点 | 方針 |
|---|---|
| **owner** | `add` / `list` は `--trip` **必須** |
| **ID 指定** | `show` / `update` / `delete` は **Participant ID**（グローバル） |
| **`--json`** | `list` / `show` で対応（既存エンティティと同型） |
| **`--sort-order`** | `add` / `update` で任意 |
| **Expense 連携** | v2 では `expense` 側オプション **追加しない** |

オプション詳細・エラーメッセージ・help 文言は **#9 Implementation Plan** へ。

---

## Export / Import Design Outline

**本フェーズでは export schema を変更しない。** v2.0.0 実装（#10）時の設計骨格:

### schema version

```text
schema_version: 4
```

### 配置

```text
top-level participants[]  — Trip 直下の兄弟（推奨・確定）
```

[participant-model.md](participant-model.md) の案を踏襲。Day / Itinerary ネストより自然。

### Export JSON 案

```json
{
  "schema_version": 4,
  "trip": {
    "name": "沖縄 瀬底 4日間",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29"
  },
  "participants": [
    { "name": "知弘", "sort_order": 0 },
    { "name": "節子", "sort_order": 1 }
  ],
  "days": [],
  "checklist_items": [],
  "notes": []
}
```

| 論点 | 方針 |
|---|---|
| **内部 ID** | export **しない** |
| **安定キー** | import 時は配列順 + `sort_order` + `name` で復元 |
| **v3 互換** | v3 export import **継続**。`participants` 省略 = 空 |
| **v4 → v3** | v3 importer は v4 を受け付けない — 想定どおり |

### Import 順序（案）

```text
1. Trip 作成
2. Participants INSERT（新 ID 採番）
3. Days / Itineraries / …
4. Expenses（paid_by_name のみ — Participant 解決なし）
```

### unknown / missing participants

| 状況 | 方針 |
|---|---|
| v4 export に `participants` キーなし | 空配列として扱う |
| v3 export を v4 importer で読む | `participants: []` |
| import 後 Expense の `paid_by_name` が Participant に無い | **エラーにしない**（v2/v3 まで） |

### sort_order 保存

export / import ともに `sort_order` を **そのまま** 保持。配列順だけに依存しない（Note / Expense と同型）。

---

## JSON Output Considerations

将来 `participant list --json` / `participant show --json` の構造案（**固定は #9 以降**）。

### `participant list --trip 1 --json`

```json
{
  "trip_id": 1,
  "participants": [
    {
      "id": 1,
      "trip_id": 1,
      "name": "知弘",
      "sort_order": 0,
      "created_at": "2026-06-19 12:00:00",
      "updated_at": "2026-06-19 12:00:00"
    }
  ]
}
```

### `participant show 1 --json`

単一 Participant オブジェクト（上記配列要素と同型）。

内部仕様扱い — [README.md](../../README.md) の `--json` 方針に従う。

---

## Validation Rules

| # | ルール | 層 | エラー例（英語域） |
|---|---|---|---|
| 1 | `name` は **必須**（add 時） | CLI | `name is required` |
| 2 | `name` trim 後 **空文字禁止** | CLI + DB NOT NULL | `name must not be empty` |
| 3 | 同一 Trip 内 **同名を許可** | 設計 | —（区別は `id`） |
| 4 | `sort_order` **負数禁止** | CLI | `sort_order must be non-negative` |
| 5 | `sort_order` 重複 | **許可** | tie-break: `id` |
| 6 | `trip_id` 存在確認 | CLI add/list | `Trip not found: N` |
| 7 | Participant ID 存在確認 | show/update/delete | `Participant not found: N` |
| 8 | 削除済み Trip | Participant は cascade 済み — 孤児なし | — |

### `name` 最大長（案）

```text
200 文字（trim 後）— Summary / Note より短い表示名向け
```

実装時定数 `MAX_PARTICIPANT_NAME_LEN` に集約（#9）。

---

## v2.0.0 Scope

### 本 Entity Design で確定するもの

| 項目 | 内容 |
|---|---|
| テーブル | `participants`（6 列） |
| 親 | `trip_id` のみ |
| フィールド | `id`, `trip_id`, `name`, `sort_order`, `created_at`, `updated_at` |
| cascade | Trip 削除で全 Participant 削除 |
| 他 entity への FK | **なし** |
| export 骨格 | v4 top-level `participants[]` |

### 実装フェーズ（#9–#10）で行うもの

- migration / `src/participant.rs` / CLI / export v4 / tests

---

## Deferred Scope

### v3 Shared Expense

```text
expenses.paid_by_participant_id
expense_beneficiaries / share ratio
Settlement 計算
Participant 削除制約（Expense 参照時）
export: paid_by_participant_ref on expenses[]
```

### v5 Travel Book

```text
しおりの同行者セクション
Participant 一覧の整形表示
```

### v6 Travel Journal

```text
Photo / Journal と Participant の紐づけ
```

### その他 Deferred

```text
memo / role 列
Checklist assigned_participant_id
Reservation guest_participant_id
User / contact book / permissions / cloud sync
```

---

## Open Questions

Implementation Plan（#9）で解決:

| # | 質問 |
|---|---|
| 1 | `participant add` 時のデフォルト `sort_order` — `0` 固定 vs 既存 max+1 |
| 2 | export v4 に `created_at` / `updated_at` を含めるか |
| 3 | `trip duplicate` で Participant を複製する際の `name` / `sort_order` の完全一致でよいか |
| 4 | `participant list` テキスト出力の列（ID / name / sort_order） |
| 5 | v3 同名 Participant 許可時の UX（一覧で区別表示するか） |
| 6 | canonical sample への Participant 投入タイミング（#10 後の任意タスク） |

---

## Completion Criteria

Issue #8（Entity Design）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `participant-entity-design.md` が存在する | 本書 |
| 2 | `participant-model.md` と整合 | §Source Responsibilities Review |
| 3 | テーブル・フィールド・関係・検証が整理 | §Table Design–§Validation |
| 4 | v2 scope / deferred が明確 | §v2.0.0 Scope, §Deferred |
| 5 | v3 Expense 境界が明確 | §Expense, §Deferred v3 |
| 6 | 実装なし | 本フェーズ対象外 |

---

## Next phase notes（Implementation Plan #9）

#9 で確定する主な項目:

- migration ファイル名・`db.rs` 組み込み
- `src/participant.rs` 関数一覧とテスト矩阵
- export v4 フィールド正式定義と `validate-export` ルール
- CLI オプション詳細・エラーメッセージ一覧
- `trip duplicate` / import 順序の手順書

Release は #12（v2.0.0）。
