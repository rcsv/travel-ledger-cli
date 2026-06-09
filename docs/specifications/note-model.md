# Note モデル（設計草案）

Caglla CLI / 将来 Web 版に向けた **Note エンティティ**（Long-form Note）の仕様メモです。  
v1.3.0 で **DB / CLI 基本 CRUD を実装済み**。Export schema v2 は未対応。

**責務の全体像**（Summary / Remark / Note / Reservation の使い分け）は [Travel Ledger Responsibilities](travel-ledger-responsibilities.md) を正とします。**Summary** の詳細は [Summary Responsibilities Review](summary-responsibilities-review.md)。**Reservation** の詳細は [Reservation モデル](reservation-model.md) を参照。本書は **Long-form Note entity** に焦点を当てます。

関連: [Day モデル](day-model.md) / [Itinerary モデル](itinerary-model.md) / [Travel Ledger Responsibilities](travel-ledger-responsibilities.md)

---

## 1. Note の目的

Note は、旅行計画・実行・振り返りにおける **自由記述の記録** を、Trip / Day / Itinerary の各階層に付与するための共通モデルです。

| 用途例 | 想定 owner |
|---|---|
| 旅全体のしおり・方針・注意事項 | Trip |
| 日別まとめ・その日の振り返り | Day |
| 訪問記録・駐車場・営業時間・現地メモ | Itinerary |

将来の Photo / Expense / Itinerary 配下 Checklist とは **別エンティティ** として扱い、所有者モデル（`owner_type` + `owner_id`）のパターンを先に確立します。

### 既存 `itinerary_items.note` との関係

現行 DB には Itinerary 行に **短い備考列** `itinerary_items.note`（TEXT, 任意）が既にあります。ユーザー向けラベルは **備考（Remark）** とし、Long-form Note とは別責務です（[travel-ledger-responsibilities.md §3](travel-ledger-responsibilities.md#3-itinerary-remarkitinerary_itemsnote)）。

| 項目 | `itinerary_items.note`（Remark） | Note モデル（Long-form） |
|---|---|---|
| 粒度 | 予定 1 件に 1 フィールド | 予定 1 件に **複数** Note 可 |
| 用途 | 旅程表の短い補足 | 検討・記録・振り返りなど長文 |
| 構造 | 単一文字列 | `title` + `body` |
| 対象 | Itinerary のみ | Trip / Day / Itinerary |
| Export | schema v3 の itinerary `note` | schema v2+ の `notes[]` |

**方針:** v1.x では両者を **併存** させる。`itinerary_items.note` の削除・統合は将来検討。Summary（Trip/Day 概要）は別フィールドとして将来追加する。

### 将来エンティティとの関係

```text
Trip
 ├─ Note / Photo          ← 本仕様: Note を先行
 ├─ Checklist             ← 現行: Trip 配下のみ
 └─ Day
      ├─ Note / Photo
      └─ Itinerary
           ├─ Expense      ← v1.x では未着手（Participant / Currency 等が重い）
           ├─ Checklist
           ├─ Note / Photo
           └─ Location     ← 現行: itinerary_items.location 列
```

Photo は Note の `body` や別テーブルと組み合わせる想定。Expense より先に Note の **所有者・削除・Export** の型を固める。

---

## 2. 対象階層

Note が付与できる owner は次の 3 種類に限定します。

```text
Trip Note       owner_type = trip       owner_id = trips.id
Day Note        owner_type = day        owner_id = days.id
Itinerary Note  owner_type = itinerary  owner_id = itinerary_items.id
```

### owner 解決の原則

- **Day Note** の CLI 指定は `--trip <trip_id> --day <day_number>` とし、内部で `days.id` に解決する（Itinerary の `--day N` / `day_id` パターンに揃える）。
- **Itinerary Note** の CLI 指定は `--itinerary <itinerary_id>`（グローバル ID）。
- Trip / Day / Itinerary が同一 DB に存在し、Day / Itinerary が指定 Trip に属することを **アプリケーション側で検証** する。

### Day Swap との関係

v1.2.0 の `day swap` は **Itinerary の `day_id` のみ** 入れ替えます。

- Day Note は `days.id` に紐づくため、**Day 番号に固定** され swap の影響を受けない。
- 意図: 「2 日目の日記」は Day 2 に残り、「2 日目に実行した予定のメモ」は Itinerary について swap で移動する。

---

## 3. DB 設計案

### 第一候補: 単一 `notes` テーブル

```sql
CREATE TABLE notes (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    owner_type  TEXT NOT NULL,
    owner_id    INTEGER NOT NULL,
    title       TEXT,
    body        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    CHECK (owner_type IN ('trip', 'day', 'itinerary'))
);

CREATE INDEX IF NOT EXISTS idx_notes_owner
    ON notes(owner_type, owner_id);
```

### カラム検討

| カラム | 推奨 | 理由 |
|---|---|---|
| `title` | **任意（NULL 可）** | 現地メモ・一行メモは title なしで追加しやすい。一覧では body 先頭行を省略表示する fallback 可 |
| `body` | **NOT NULL** | 中身のない Note 行は意味が薄い。空文字 `""` は CLI 側で **拒否**（DB 制約は NOT NULL のみ） |
| `owner_type` | **CHECK 制約 + Rust enum** | `'trip' \| 'day' \| 'itinerary'`。将来 Photo 等は別テーブルまたは owner_type 拡張を別フェーズで検討 |
| `owner_id` | **FK なし（案 C）** | ポリモーフィック関連。詳細は §4 |
| `sort_order` | **v1.3.0 では追加推奨** | 同一 owner 内の並び替え用。初期値 `0`。未指定時は `sort_order → id` で表示（Checklist と同型） |
| `created_at` / `updated_at` | **既存と同形式** | `YYYY-MM-DD HH:MM:SS`（`db::now_string()` と同一）。タイムゾーンはローカル |

### 削除時の Note 扱い（案 C 前提）

| トリガー | Note の扱い |
|---|---|
| `trip delete` | 当該 Trip 配下の **すべて** の Note を削除（Trip / Day / Itinerary 由来すべて） |
| Day 行削除（`trip update` で期間短縮） | 当該 `days.id` の Day Note を削除 |
| `itinerary delete` | 当該 `itinerary_items.id` の Itinerary Note を削除 |
| `day swap` | Note は **変更しない** |

実装は `delete_notes_for_trip` / `delete_notes_for_day` / `delete_notes_for_itinerary` 等の関数 + 既存 delete 処理からの呼び出し。

---

## 4. 外部キー / cascade 方針

### 案の比較

| 案 | 概要 | メリット | デメリット |
|---|---|---|---|
| **A** | FK なし、アプリ検証のみ | 共通テーブル・拡張容易 | 孤児 Note を DB だけでは防げない |
| **B** | `trip_notes` / `day_notes` / `itinerary_notes` | FK + ON DELETE CASCADE が自然 | テーブル・CLI 重複、拡張で増殖 |
| **C** | 共通 `notes` + **手動 cascade** | 共通モデル + 実装量抑制 | 削除経路のテストが重要 |

### 推奨: **案 C（共通 notes + 手動 cascade）**

現行 CLI も `days` は Trip 削除時に **明示的な cascade 関数** で扱っており（`itinerary_items` / `checklist_items` は FK CASCADE）、Note も同じ思想で統一する。

#### 孤児 Note 対策

- create / update 時: owner 存在チェック必須
- delete 時: 関連 Note を必ず同時削除（トランザクション推奨）
- 将来: `db doctor` 的な整合性チェックコマンドは任意（v1.3.0 必須ではない）
- migration 時: 既存 DB に `notes` 追加のみ（backfill 不要）

#### owner_id の参照先

| owner_type | owner_id が指すテーブル |
|---|---|
| `trip` | `trips.id` |
| `day` | `days.id` |
| `itinerary` | `itinerary_items.id` |

---

## 5. CLI 設計案

### コマンド一覧（Phase 3 想定）

```bash
note add --trip 1 --title "全体メモ" --body "..."
note add --trip 1 --day 3 --body "2日目の振り返り"
note add --itinerary 12 --title "駐車場" --body "..."

note list --trip 1
note list --trip 1 --day 3
note list --itinerary 12

note show 1
note update 1 --title "..." --body "..."
note delete 1
```

JSON:

```bash
note list --trip 1 --json
note show 1 --json
```

### CLI 設計の推奨

| 論点 | 推奨 |
|---|---|
| `--trip` / `--day` / `--itinerary` | **`note add` / `note list` では排他**（1 つ必須）。`--day` は `--trip` と **セット必須** |
| `note list` の owner 指定 | **必須**。owner 未指定はエラー |
| `note list --trip 1` の範囲 | **v1.3.0 デフォルト: Trip 直下の Note のみ**。Day / Itinerary 配下を含めるのは `--all`（将来） |
| `note add` の title | **省略可**（NULL） |
| `note add` の body | **`--body` 必須**（空文字不可）。将来 `--editor` / stdin は別フェーズ |
| `note show` / `update` / `delete` | **Note ID** で指定（Trip ID ではない） |
| エラーメッセージ | 既存に合わせ英語ドメイン（例: `Note not found: 1`）+ 日本語説明は README |

### `note list` 出力イメージ（テキスト）

```text
Trip 1 の Note (2 件):

ID  Title        Body (先頭)
1   全体メモ     レンタカーは...
2   (なし)       空港からはモノレール...
```

### JSON 出力（内部仕様）

`note list --json`（owner 指定時）:

```json
{
  "owner_type": "trip",
  "owner_id": 1,
  "notes": [
    {
      "id": 1,
      "owner_type": "trip",
      "owner_id": 1,
      "title": "全体メモ",
      "body": "...",
      "sort_order": 0,
      "created_at": "...",
      "updated_at": "..."
    }
  ]
}
```

`note show --json`: 単一 Note オブジェクト。

---

## 6. Export / Import 方針

**v1.3.0 Phase 1 では Export 実装不要。** 方針のみ整理する。

### Note を export 対象に含めるべきか

**含めるべき**（バックアップ・Web 移行の観点）。ただし **schema v1 への後付けは避け**、**schema v2** で正式に追加する。

### schema v2 案（概要）

```json
{
  "schema_version": 2,
  "generator": "caglla-cli",
  "generator_version": "1.3.0",
  "exported_at": "...",
  "trip": { },
  "itinerary_items": [ ],
  "checklist_items": [ ],
  "notes": [
    {
      "owner_type": "day",
      "owner_day_number": 2,
      "title": null,
      "body": "...",
      "sort_order": 0
    },
    {
      "owner_type": "itinerary",
      "owner_itinerary_ref": { "day": 1, "title": "首里城" },
      "body": "..."
    }
  ]
}
```

| 論点 | 方針 |
|---|---|
| v1 との互換 | v1 export / import は **現行どおり**（Note なし）。v2 export は v2 import のみ保証 |
| owner 参照 | export JSON では **安定参照** を優先: Trip → `trip.id` 相当、Day → `day_number`、Itinerary → export 時 ID または `(day, title, start_time)` 等（import 時に再解決） |
| import 復元 | Trip / Day 作成後に Note を INSERT。Itinerary Note は import 後の新 ID にマッピング |
| `validate-export` | v2 対応時に `notes[]` 検証を追加 |
| `trip diff` | v2 以降で `notes` セクション比較を追加（v1.3.0 必須ではない） |
| `export-md` | v1.3.0 必須ではない（§7） |

**推奨リリース分割:** v1.3.0 = Note DB + CLI。Export schema v2 = **v1.4.0 以降** の別フェーズでもよい（実装量とテストを分離）。

---

## 7. Markdown Export との関係

v1.3.0 での `trip export-md` への組み込みは **未定（必須ではない）**。

含める場合の表示案:

```markdown
# Trip Name

## Trip Notes

### 全体メモ
レンタカーは...

## Day 1 - 2026-04-26

### Day Notes
- 早朝出発

### Itinerary
09:00 首里城
#### Notes
- 駐車場は付近の有料P
```

| 論点 | 方針 |
|---|---|
| 並び | Trip Notes → Day ごと（Day Notes → Itinerary → Itinerary Notes） |
| title なし Note | `###` 見出しなしで body のみ、または `(メモ)` 等の固定見出し |
| `itinerary_items.note` | Markdown では Itinerary 行の直下に「メモ: ...」として **従来どおり** 表示。Note エンティティとは別節 |

---

## 8. Doctor / Stats との関係

| 機能 | v1.3.0 |
|---|---|
| `trip doctor` | **原則対象外**（Note 不足は warning にしない） |
| `trip stats` | **集計対象外**（件数表示もしない） |
| `trip advisor` | **対象外** |

将来例: 「重要カテゴリの Itinerary に Note がない」等は **optional suggestion** として拡張可能。v1.3.0 では実装しない。

---

## 推奨案サマリー

| 項目 | 推奨 |
|---|---|
| テーブル | 単一 `notes` |
| 外部キー | **案 C** — FK なし、アプリ側 cascade |
| `title` | 任意 |
| `body` | NOT NULL、CLI で空文字拒否 |
| `sort_order` | あり（デフォルト 0） |
| owner | `trip` / `day` / `itinerary` + `owner_id` |
| Day 解決 | CLI `--trip` + `--day` → `days.id` |
| Export | schema **v2** で `notes[]`（実装は v1.3.0 または次フェーズ） |
| Markdown | v1.3.0 必須ではない |

---

## 実装フェーズ（参考）

| Phase | 内容 | v1.3.0 スコープ |
|---|---|---|
| 1 | 本仕様メモ | **今回** |
| 2 | DB / Model / CRUD | 想定 |
| 3 | CLI + `--json` | 想定 |
| 4 | cascade / 整合性テスト | 想定 |
| 5 | README / release notes | 想定 |
| — | Export schema v2 | **別フェーズ推奨** |
| — | export-md / diff / doctor | 将来 |

---

## 実装に進む場合のリスク

| リスク | 内容 | 緩和 |
|---|---|---|
| `itinerary_items.note` との混同 | ユーザー・ドキュメントで二種類の「メモ」が存在 | README / CLI help で役割を明示 |
| 孤児 Note | FK なしのため削除漏れ | 全 delete 経路のテスト、transaction |
| Day 期間短縮 | Day 削除時の Note 漏れ | `sync_days_to_trip_duration` から cascade 呼び出し |
| Export 参照の不安定さ | Itinerary ID は import 後に変わる | v2 では day_number + 属性による参照を検討 |
| スコープ膨張 | export-md / v2 / doctor を同時に入れる | v1.3.0 は DB + CLI に限定 |
| `note list --all` | 階層横断一覧の仕様が複雑 | v1.3.0 では Trip 直下のみ、後続で `--all` |

---

## 参照

- 現行モデル: [day-model.md](day-model.md)
- DB 初期化: `src/db.rs`
- Export 構造: `src/models.rs`（`TripExport`）, `src/trip.rs`
- ユーザー向け: [command-reference.md](../command-reference.md) の Note 節
