# Export / Import

JSON エクスポート・インポート・検証・比較の手順です。スキーマ定義は [specifications/export-schema.md](specifications/export-schema.md) を参照してください。

## JSON エクスポート（trip export）

旅行 1 件と、紐づく日程・チェックリストを JSON で出力します。将来の Web 版や Firebase / Firestore への移行を想定した形式です。

**export / import の対象:** **Trip**、**Itinerary（`itinerary_items` / `days[]`）**、**Checklist**、**Note（schema v2+）**、**Expense（schema v3+）**、**Reservation（schema v3+）**、**Participant（schema v4+）** です。`trip export` → `db reset` → `trip import` でバックアップ／リストアできます。

現行 export は **`schema_version: 5`** です。Import は **v1 / v2 / v3 / v4 / v5** に対応します（v3 以前の JSON に `participants` がなくても import 可能）。スキーマ詳細は [specifications/export-schema.md](specifications/export-schema.md) を正としてください。

Export JSON には **`schema_version`**、**`generator`**（`caglla-cli`）、**`generator_version`**、**`exported_at`**（RFC3339）が含まれます。旧形式（メタデータ省略）とも後方互換です。

```bash
# 標準出力に表示
cargo run -- trip export 1

# ファイルに保存
cargo run -- trip export 1 --output backup.json
```

出力例（構造 — **現行 v5**）:

```json
{
  "schema_version": 5,
  "generator": "caglla-cli",
  "generator_version": "3.0.0",
  "exported_at": "2026-06-07T00:00:00Z",
  "trip": {
    "name": "沖縄旅行",
    "start_date": "2026-04-26",
    "end_date": "2026-04-29",
    "summary": null
  },
  "days": [
    {
      "day_number": 1,
      "summary": null,
      "itineraries": [
        {
          "title": "首里城",
          "sort_order": 0,
          "start_time": "09:00",
          "duration_minutes": 90,
          "travel_minutes": 20,
          "location": "沖縄県那覇市首里金城町1-2",
          "expenses": [],
          "reservations": []
        }
      ]
    }
  ],
  "checklist_items": [
    {
      "title": "パスポート",
      "is_done": false,
      "sort_order": 0
    }
  ],
  "notes": [
    {
      "owner_type": "trip",
      "title": "全体メモ",
      "body": "..."
    }
  ],
  "participants": [
    {
      "name": "ともさん",
      "sort_order": 0,
      "is_self": true
    },
    {
      "name": "妻",
      "sort_order": 1,
      "is_self": false
    }
  ]
}
```

現行 export では Itinerary は **`days[].itineraries[]`** にネストします（top-level `itinerary_items` は v2 まで）。内部 DB id（`trip.id` / `itinerary_id` 等）は export しません。

Itinerary は **日目 → 並び順（`sort_order`）** でソートされた状態で出力されます（Sequence-first — [ordering-model.md](specifications/ordering-model.md)）。`checklist_items` は一覧表示と同じく、未完了 → 完了済み、同状態内では `sort_order` → `id` の順で出力されます。

**Participant（v4+）:** top-level `participants[]` に Trip スコープの参加行を出力します。`is_self` は同一 Trip で最大 1 件。v3 export を import する場合、`participants` キーがなくても **空配列として復元** されます。

**Shared Expense（v5）:** Expense オブジェクトに optional な `paid_by_participant_ref` と `beneficiaries[]`（`participant_ref` = `participants[].name`）を出力します。省略時は **personal expense**（v4 import 互換）。同名 Participant がある Trip では ref が曖昧になるため import / validate-export は **拒否** します。

### 旧フォーマットとの互換

Import は次の旧形式も読み込めます（**ただし `trip.start_date` / `trip.end_date` は必須**）。

| 旧形式 | 扱い |
|---|---|
| `trip.start_date` / `trip.end_date` なし | **import 不可** |
| `schema_version` 未指定 / `1` | v1 形式として import（`notes` なし） |
| `schema_version: 2` | v2 形式として import（`notes` あり、flat `itinerary_items`） |
| `schema_version: 3` | v3 形式（nested `days[]`、Expense / Reservation）。**`participants` 省略可** → 空配列 |
| `schema_version: 4` | v4 形式（v3 + `participants[]`）。multiple `is_self: true` は **import / validate 拒否** |
| `schema_version: 5` | v5 形式（v4 + Expense `paid_by_participant_ref` / `beneficiaries[]`）。省略フィールドは personal。**同名 Participant + ref** は **import / validate 拒否** |
| `schema_version` / `generator` / `generator_version` / `exported_at` なし | メタデータなしとして import（問題なし） |
| `generator: "unknown"` や旧 `generator_version` | import 可能（warning なし） |
| `checklist_items` なし（v1.0.2 以前） | チェックリストは空として import |

## JSON インポート（trip import）

`export` で出力した JSON を読み込み、**新しい Trip として**登録します。

```bash
cargo run -- trip import backup.json
```

| 動作 | 説明 |
|---|---|
| ID の扱い | JSON 内の `id` / `trip_id` は無視し、DB の AUTOINCREMENT で新規採番 |
| trip_id の変換 | 日程・チェックリストの `trip_id` は、新しく作成された Trip の ID に置き換わる |
| 日時 | `created_at` / `updated_at` はインポート時に新しく設定される |
| 旅行期間 | `trip.start_date` / `trip.end_date` は必須。import 時に Day 1..N を自動生成 |
| Checklist | `checklist_items` があれば復元する。省略時は空配列として扱う |
| Note | `notes` があれば復元する（schema v2+）。省略時は空配列として扱う |
| Expense / Reservation | schema v3+ の nested `days[].itineraries[]` 配下から復元 |
| **Participant** | schema v4+ の `participants[]` から復元。**v3 JSON でキー省略時は空配列** |
| **Shared Expense** | schema v5 の Expense ref から payer / beneficiaries を復元。**v4 以前は省略 = personal** |
| メタデータ | `schema_version` / `exported_at` は import 時に無視される |

**import 後の Trip ID について:** export JSON 内の `trip.id` は、import 後の DB 上の ID を保証しません。import 完了サマリーに表示される ID を使ってください。

エクスポートとインポートの流れ:

```bash
cargo run -- trip export 1 --output backup.json
cargo run -- trip validate-export backup.json
cargo run -- trip import backup.json
```

## export ファイルの検証（trip validate-export）

export ファイルが **import 可能か** を import 前に確認します。DB は使わず、ファイルのみを読み込みます。

```bash
cargo run -- trip validate-export backup.json
cargo run -- trip validate-export backup.json --json
```

| 用語 | 意味 |
|---|---|
| `valid` | import 可能か（`errors` が空） |
| `warnings` | 推奨形式との差異や注意点（import 可能でも表示される） |
| `checks` | export 形式としての確認結果（✓/✗） |

終了コード: `valid: true` → exit 0 / `valid: false` またはファイル読込エラー → exit 1。

**v4 Participant 検証:** 同一 Trip に `is_self: true` が 2 件以上ある export は **`valid: false`** になります（import も拒否）。

**v5 Shared Expense 検証:** `paid_by_participant_ref` / `beneficiaries[].participant_ref` が Trip 内 Participant に一意に解決できない export は **`valid: false`** になります。v4 ファイルは v5 専用 ref 検査を **スキップ** します。詳細は [export-schema.md](specifications/export-schema.md) §validate-export。

## 旅行 JSON の比較（trip diff）

2 つの `trip export` JSON を比較し、Trip・Itinerary・Note・Summary・Reservation・**Participant（v4+）**・**Expense payer/beneficiaries（v5+）** の差分を表示します。

```bash
cargo run -- trip diff trip-old.json trip-new.json
```

比較対象:

| 種別 | 表示例 |
|---|---|
| Trip | `- name: 旧名` / `+ name: 新名` / summary 変更 |
| Itinerary | `- Day1 09:00 首里城` / `+ ...` / `~ ...`（フィールド変更） |
| Note | `- Note removed: Day 2 / 夕食候補` / `+ Note added: Trip / 持ち物メモ` |
| Summary | Trip / Day summary の追加・削除・変更 |
| Reservation | added / removed / modified |
| Participant | `Participants:` 見出し下に added / removed / `is_self` changed |
| Expense (v5+) | added / removed / `payer` or `beneficiaries` modified（両 export が schema v5+ の場合） |

v1 export（`notes` なし）と v2 export（`notes: []`）を比較しても異常終了しません。v3 export に `participants` が無い場合は空配列として比較します。v4 同士の比較では Expense の payer/beneficiaries は比較しません。

## JSON 出力について

一部の read 系コマンドは `--json` に対応しています。ツール連携・自動化向けです。**内部仕様扱い**（構造は将来変更の可能性あり）。`trip doctor --json` と `trip advisor --json` は **v1.0.6 以降の構造化フォーマット**（`schema_version: 1`）を使います。

`--json` 指定時は人間向けの見出しや説明文を出さず、pretty JSON のみ stdout に出力します。

| コマンド | 例 |
|---|---|
| `trip list` | `cargo run -- trip list --json` |
| `trip show` | `cargo run -- trip show 1 --json` |
| `trip stats` | `cargo run -- trip stats 1 --json` |
| `trip doctor` | `cargo run -- trip doctor 1 --json` |
| `trip advisor` | `cargo run -- trip advisor 1 --json` |
| `trip validate-export` | `cargo run -- trip validate-export backup.json --json` |
| `day list` | `cargo run -- day list 1 --json` |
| `day show` | `cargo run -- day show 1 2 --json` |
| `note list` | `cargo run -- note list --trip 1 --json` |
| `note show` | `cargo run -- note show 1 --json` |
| `itinerary list` | `cargo run -- itinerary list 1 --json` |
| `itinerary show` | `cargo run -- itinerary show 1 --json` |
| `checklist list` | `cargo run -- checklist list 1 --json` |
| `checklist show` | `cargo run -- checklist show 1 --json` |
| `expense list` | `cargo run -- expense list --trip 1 --json` |
| `expense show` | `cargo run -- expense show 1 --json` |

### trip doctor / advisor JSON（v1.0.6+）

**破壊的変更:** v0.9.3 以前の `trip doctor --json` は issue 配列を root に出力していました。v1.0.6 以降は envelope オブジェクトです。

#### Doctor envelope

```json
{
  "schema_version": 1,
  "trip_id": 1,
  "issues": [
    {
      "code": "no_restaurant",
      "severity": "warning",
      "message": "Day 3 has no restaurant",
      "target": { "type": "day", "id": 3 },
      "details": { "day": 3 }
    }
  ]
}
```

#### Advisor envelope

Doctor と同じ issue フィールドに加え、各 issue に `advice`（必須）と `commands`（`--with-commands` 時のみ）があります。

#### Issue フィールド

| フィールド | 説明 |
|---|---|
| `code` | 安定 ID: `empty_itinerary`, `overloaded_day`, `no_restaurant`, `high_travel_time`, `missing_duration` |
| `severity` | `info` または `warning` |
| `message` | 人間向けテキスト |
| `target` | 問題の対象 |
| `details` | code ごとの構造化メタデータ |

#### `target.id` の意味

| `target.type` | `target.id` の意味 |
|---|---|
| `trip` | 旅行 ID（`trips.id`） |
| `day` | 旅行内の日数（1-based。**DB の day 行 ID ではない**） |
| `itinerary` | 日程 ID（`itinerary_items.id`） |
