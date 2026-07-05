# Export Schema

Caglla CLI の trip export / import JSON 形式。

**外向き入口（Travel Ledger 公開契約）:** [public/schema.md](../public/schema.md) — 現行 canonical は **`schema_version: 8`**。CLI パッケージ version（`Cargo.toml`）とは独立。

**本ドキュメントの範囲:** JSON の **構造・バージョン・検証ルール** のみ。Itinerary / Expense / Note の **意味論・責務** は各モデル仕様を参照してください。

| トピック | 参照 |
|---|---|
| Itinerary が何を表すか | [Itinerary モデル](itinerary-model.md) |
| Expense の親子関係 | [Expense モデル](expense-model.md) |
| Note の owner 解決 | [Note モデル](note-model.md) |
| Reservation（将来） | [Reservation Implementation Plan](reservation-implementation-plan.md) §7 |
| Summary | [Summary Responsibilities Review](summary-responsibilities-review.md) §9 |
| Participant（v4） | [participant-implementation-plan.md](participant-implementation-plan.md) |
| Day と `day_number` | [Day モデル](day-model.md) |
| 並び順（`sort_order` / `start_time`） | [Ordering モデル](ordering-model.md) |

---

## Schema versions

| `schema_version` | 状態 | 説明 |
|---|---|---|
| 未指定 | v1（effective） | 旧形式。`notes` なし |
| `1` | v1 | 明示的 v1。`notes` なし |
| `2` | v2 | Note を含む（`itinerary_items` フラット） |
| `3` | v3 | Note + **nested Expense**（`days[]`） |
| `4` | v4 | v3 + top-level **`participants[]`**（`is_self` 含む） |
| `5` | v5 | v4 + Expense **`paid_by_participant_ref`** / **`beneficiaries[]`** — [shared-expense-entity-design.md](shared-expense-entity-design.md) |
| `6` | v6 | v5 + nested **`estimates[]`**（Planned Budget）— [estimate-entity-design.md](estimate-entity-design.md) |
| `7` | v7 | v6 + top-level **`receipts[]`**（Receipt Inbox metadata — Trip level） |
| `8` | v8（**現行 export**） | v7 + Receipt **`trashed_at`**（RFC3339）— [v3.7.0 plan](v3.7.0-receipt-assignment-and-trash-implementation-plan.md) |

Import 時の解釈:

- `schema_version` 未指定 → v1
- `schema_version: 1` → v1
- `schema_version: 2` → v2
- `schema_version: 3` → v3（`participants` 省略時は空配列扱い）
- `schema_version: 4` → v4
- `schema_version: 5` → v5
- `schema_version: 6` → v6
- `schema_version: 7` → v7
- `schema_version: 8` → v8

v1 / v2 / v3 / v4 / v5 / v6 / v7 export は引き続き import 可能です。現行 CLI の `trip export` は **`schema_version: 8`** を出力します。

## Top-level structure (v6)

v5 と同一の top-level 構造。Itinerary オブジェクトに optional な `estimates[]` が追加されます（下記）。

### Estimate オブジェクト（v6）

`days[].itineraries[].estimates[]` にネストします。

| フィールド | 必須 | 説明 |
|---|---|---|
| `title` | 任意 | 見積ラベル |
| `amount` | はい | 最小通貨単位の整数（Expense export と同型） |
| `currency` | はい | ISO 4217 通貨コード |
| `note` | 任意 | 補足 |
| `sort_order` | はい | Itinerary 内の並び順 |

export 時 **`id` / `created_at` / `updated_at` は出力しません**（再 import で新 ID 採番）。

v5 import 互換: `estimates` 省略 = 空配列。

## Top-level structure (v5)

v4 と同一の top-level 構造。Expense オブジェクトに optional フィールドが追加されます（下記）。

### Expense shared fields (v5)

| フィールド | 必須 | 説明 |
|---|---|---|
| `paid_by_participant_ref` | 任意 | payer の Participant 参照（`participants[].name` と完全一致） |
| `beneficiaries` | 任意 | `{ "participant_ref": "名前", "sort_order": 0 }` の配列。省略 = personal |

## Top-level structure (v4)

```json
{
  "schema_version": 4,
  "generator": "caglla-cli",
  "generator_version": "1.22.0",
  "exported_at": "2026-06-07T00:00:00Z",
  "trip": {},
  "days": [],
  "checklist_items": [],
  "notes": [],
  "participants": []
}
```

### Participants (v4)

| フィールド | 必須 | 説明 |
|---|---|---|
| `name` | はい | 表示名 |
| `sort_order` | はい | Trip 内の並び順 |
| `is_self` | はい | この Trip における自分マーカー（同一 Trip で最大 1 件） |

internal `id` は export しません。v3 import では `participants` 省略 = 空配列。

## Top-level structure (v3)

```json
{
  "schema_version": 3,
  "generator": "caglla-cli",
  "generator_version": "1.6.0",
  "exported_at": "2026-06-07T00:00:00Z",
  "trip": {},
  "days": [],
  "checklist_items": [],
  "notes": []
}
```

| フィールド | 必須 | 説明 |
|---|---|---|
| `schema_version` | export 時は付与 | 未指定 import は v1 扱い |
| `trip` | ✓ | Trip 本体（`start_date` / `end_date` 必須） |
| `days` | ✓ | Day ごとの Itinerary 一覧（Expense は各 Itinerary 配下） |
| `checklist_items` | export 時は付与 | チェックリスト（空配列可） |
| `notes` | export 時は付与 | Note 一覧（空配列可） |

v3 では **top-level `itinerary_items` を使いません**。Itinerary は `days[].itineraries[]` にネストします。

### Day / Itinerary / Expense (v3)

v3 の Itinerary オブジェクトは **行動のスナップショット** です。フィールドの意味（`title` 必須、`location` 任意、Itinerary 直付け費用なし等）は [Itinerary モデル](itinerary-model.md) を正とします。以下は **JSON 構造の例** です。

```json
{
  "day_number": 2,
  "itineraries": [
    {
      "title": "美ら海水族館",
      "sort_order": 0,
      "start_time": "09:00",
      "duration_minutes": 120,
      "travel_minutes": 30,
      "location": "沖縄県国頭郡本部町",
      "category": "activity",
      "expenses": [
        {
          "title": "入館料",
          "amount": 2500,
          "currency": "JPY",
          "paid_by_name": null,
          "expense_date": null,
          "note": null,
          "sort_order": 0
        }
      ]
    }
  ]
}
```

| 論点 | 方針（export 構造） |
|---|---|
| 内部 ID | **`trip.id` / `itinerary_id` / `expense_id` は export しない** |
| Expense の親 | **`days[].itineraries[].expenses[]` のみ**（top-level `expenses[]` は使わない）。Trip / Day 直下 Expense は存在しない — [Expense モデル](expense-model.md) |
| import 順序 | Trip → Day（自動）→ Itinerary → Checklist → Note → **Expense** |
| `amount` / `currency` | Expense で必須。`currency` は 3 文字英字（`validate_currency_code`） |
| `expense_date` | 省略可（NULL）。指定時は `YYYY-MM-DD` |
| Itinerary フィールド | `title`, `sort_order` 必須。`location` / `category` / `start_time` 等は任意 — 詳細は [Itinerary モデル](itinerary-model.md)。並び順は [Ordering モデル](ordering-model.md) |

## Top-level structure (v2)

v2 は `itinerary_items` をフラット配列として保持します。Expense は含みません。

```json
{
  "schema_version": 2,
  "generator": "caglla-cli",
  "generator_version": "1.4.0",
  "exported_at": "2026-06-07T00:00:00Z",
  "trip": {},
  "itinerary_items": [],
  "checklist_items": [],
  "notes": []
}
```

`trip` / `itinerary_items` / `checklist_items` の v1 フィールド名は変更していません（`days` / `itineraries` / `checklists` にはしていない）。

## Notes (schema v2 / v3)

Note エントリは `owner_type` タグで種別を区別します。DB の内部 id（`notes.id`、Day id、Itinerary id）は **export しません**。

### Trip Note

```json
{
  "owner_type": "trip",
  "title": "全体メモ",
  "body": "..."
}
```

Import 時は新規 Trip に紐づけます。

### Day Note

```json
{
  "owner_type": "day",
  "day_number": 2,
  "title": "2日目メモ",
  "body": "..."
}
```

Import 時は `day_number` から対象 Day を解決します。

### Itinerary Note

```json
{
  "owner_type": "itinerary",
  "itinerary_key": {
    "day_number": 2,
    "sort_order": 3,
    "start_time": "09:00",
    "title": "美ら海水族館"
  },
  "title": "水族館メモ",
  "body": "..."
}
```

Import 時の `itinerary_key` 解決優先順位:

1. `day_number` + `sort_order`
2. `day_number` + `start_time` + `title`
3. `day_number` + `title`

解決できない場合は import error です。複数一致も error です。

v3 export では Itinerary Note の `itinerary_key` 解決に、flatten 後の `days[].itineraries[]` 相当データを使用します。

## validate-export

`trip validate-export` は export JSON を **import 前に検証** します。DB は使いません。v3 / v4 export では [Note モデル](note-model.md) の owner 解決・Expense / Reservation ネスト構造も検証します。

### 共通（全 schema）

- JSON パース可能であること
- `trip.name` / `trip.start_date` / `trip.end_date` が import 可能な値であること

### v1 / v2

- `schema_version` がサポート範囲（未指定 / 1 / 2）
- `notes` が配列（キーがある場合）
- `owner_type` が有効値
- Day Note の `day_number` が旅行期間内
- Itinerary Note の `itinerary_key` 存在・解決可能性
- `title` / `body` の型（JSON 構造）

v1 / v2 では nested `days[]` / `expenses` / `participants` チェックは **非対象**（`✗ expenses` / `✗ participants` 等）。

### v3 追加検証

- `schema_version: 3`（または v4 として読み込むが `participants` 省略 = 空配列）
- `days` が配列であること
- 各 `day_number` が旅行期間内
- 各 Itinerary の `title` 必須
- nested `expenses[]` の `currency` 必須・形式検証
- nested `expenses[]` の `expense_date` 形式（指定時）
- nested `reservations[]`（存在時）の必須フィールド
- Trip / Day `summary` 長さ（存在時）

解決不能な Note は validation error です。

**v3 import 互換:** `participants` キーがなくても import 可能（空配列として扱う）。`validate-export` では v3 ファイルに `participants` が無い場合、warning のみのことがあります。

### v4 追加検証

v4 は v3 の検証に加え:

| ルール | 内容 |
|---|---|
| `participants` キー | **配列であること**（空配列可）。省略時は v3 互換として warning の可能性 |
| 各 Participant | `name` 非空、`sort_order` ≥ 0、`is_self` は boolean |
| **multiple self** | 同一 Trip で `is_self: true` は **最大 1 件**。2 件以上は **validation error**（import も拒否） |

Participant の export 検証は import 前チェックと **同一ロジック**（`collect_export_participant_validation_errors`）です。

### v5 追加検証

v5 は v4 の検証に加え、nested `expenses[]` の Shared Expense ref を検証します。

| ルール | 内容 |
|---|---|
| `paid_by_participant_ref` | 指定時 — `participants[].name` と **完全一致**で解決できること |
| `beneficiaries[].participant_ref` | 各 ref が Trip 内 Participant に解決できること |
| **同名 Participant** | 同一 Trip に同名 `participants[].name` があり ref が曖昧 → **validation error**（import も拒否） |
| v4 ファイル | v5 専用 ref 検査は **スキップ**（v4 Participant ルールは継続） |

省略時の意味: `paid_by_participant_ref` / `beneficiaries` なし → **personal expense**（v4 import 互換）。

### v6 追加検証

v6 は v5 の検証に加え、nested `estimates[]` を検証します。

| ルール | 内容 |
|---|---|
| `amount` | 必須、非負整数 |
| `currency` | 必須、非空（ISO 4217 形式） |
| v5 以前 | Estimate 専用検査は **スキップ**（`estimates` 省略 = 空） |

### v7 追加検証

v7 は v6 の検証に加え、top-level `receipts[]` を検証します。

| ルール | 内容 |
|---|---|
| `receipts` | 配列（省略 = 空）。Trip level — Itinerary 配下ではない |
| `status` | 必須（例: `unreviewed` / `ignored`） |
| `amount` / `currency` | ペアで指定する場合は両方必須 |
| v6 以前 | Receipt 専用検査は **スキップ** |

Receipt は **Actual ではない** — Expense 化待ちの未整理候補。詳細: [v3.5.0 Receipt Inbox concept](v3.5.0-receipt-inbox-concept-design.md)。

### v8 追加検証（現行 export）

v8 は v7 の検証に加え、Receipt の Trash フィールドを検証します。

| ルール | 内容 |
|---|---|
| `trashed_at` | 指定時は **RFC3339** 形式 |
| v7 以前 | `receipts` / `trashed_at` 省略可 |

現行 `trip validate-export` は **schema v8** を canonical として警告・検証します。Public JSON examples: [public/examples/](../public/examples/)。

## trip diff

`trip diff <old.json> <new.json>` は 2 つの export JSON を比較します。v3+ export は flatten 後の Itinerary / Expense / Reservation コンテキストで比較します。

| 対象 | 表示例 |
|---|---|
| Trip フィールド | `- name: 旧名` / `+ name: 新名` / Trip summary |
| Itinerary | `- Day1 09:00 首里城` / `+ ...` / `~ ...`（フィールド変更） |
| Note | `- Note removed: ...` / `+ Note added: ...` / `~ Note changed: ...` |
| Summary | Trip / Day summary の追加・削除・変更 |
| Reservation | added / removed / modified |
| **Participant (v4+)** | added / removed / `is_self` changed（キー: `sort_order` + `name`）。rename / reorder は **removed + added** として保守的に検出 |
| **Expense (v5+)** | added / removed / `payer` or `beneficiaries` modified — **両 export が schema v5+ の場合のみ** shared フィールドを比較 |
| **Estimate (v6+)** | added / removed / `amount` / `currency` / `title` / `note` / `sort_order` modified — **両 export が schema v6+ の場合のみ** 比較 |

**v4 同士の比較:** Expense の payer / beneficiaries は比較しません（shared フィールドは export に無いため）。

**v5 vs v6 の比較:** Estimate は比較しません（v5 export に `estimates` が無いため）。

v1 export（`notes` なし）と v2 export（`notes: []`）を比較しても異常終了しません。v3 同士で `participants` が無い場合は空配列として比較します。

## 将来バージョン（現 schema に含めないもの）

以下は **v8 時点では export JSON に含めません**（Participant / Shared Expense / Estimate / Receipt Inbox は **含む**）:

| 項目 | 備考 |
|---|---|
| Photo / Attachment | 製品 v6 想定 |
| **Settlement**（精算結果・transfer 計算） | v3.x — recording は v5 で `paid_by_participant_ref` / `beneficiaries[]` |
| Person / Traveler Profile（Root 正本） | 将来。v4+ Participant は Trip スコープの参加行のみ |
| Multi-currency conversion | 将来 |
| XML / XSD export | 非対象 |
