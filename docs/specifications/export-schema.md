# Export Schema

Caglla CLI の trip export / import JSON 形式。

**本ドキュメントの範囲:** JSON の **構造・バージョン・検証ルール** のみ。Itinerary / Expense / Note の **意味論・責務** は各モデル仕様を参照してください。

| トピック | 参照 |
|---|---|
| Itinerary が何を表すか | [Itinerary モデル](itinerary-model.md) |
| Expense の親子関係 | [Expense モデル](expense-model.md) |
| Note の owner 解決 | [Note モデル](note-model.md) |
| Day と `day_number` | [Day モデル](day-model.md) |
| 並び順（`sort_order` / `start_time`） | [Ordering モデル](ordering-model.md) |

---

## Schema versions

| `schema_version` | 状態 | 説明 |
|---|---|---|
| 未指定 | v1（effective） | 旧形式。`notes` なし |
| `1` | v1 | 明示的 v1。`notes` なし |
| `2` | v2 | Note を含む（`itinerary_items` フラット） |
| `3` | v3（**現行 export**） | Note + **nested Expense**（`days[]`） |

Import 時の解釈:

- `schema_version` 未指定 → v1
- `schema_version: 1` → v1
- `schema_version: 2` → v2
- `schema_version: 3` → v3

v1 / v2 export は引き続き import 可能です。現行 CLI の `trip export` は `schema_version: 3` を出力します。

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

### v1 / v2

- `schema_version` がサポート範囲（未指定 / 1 / 2 / 3）
- `notes` が配列（キーがある場合）
- `owner_type` が有効値
- Day Note の `day_number` が旅行期間内
- Itinerary Note の `itinerary_key` 存在・解決可能性
- `title` / `body` の型（JSON 構造）

v1 / v2 では `expenses` チェックは **非対象**（`✗ expenses`）です。

### v3 追加検証

- `days` が配列であること
- 各 `day_number` が旅行期間内
- 各 Itinerary の `title` 必須
- nested `expenses[]` の `currency` 必須・形式検証
- nested `expenses[]` の `expense_date` 形式（指定時）

解決不能な Note は validation error です。

## trip diff (v1.4.1+)

`trip diff` は export JSON 2 件を比較し、`notes[]` の差分を表示します。

| 表示 | 意味 |
|---|---|
| `+ Note added: ...` | 新側にのみ存在 |
| `- Note removed: ...` | 旧側にのみ存在 |
| `~ Note changed: ...` | 同一キーで `body` または（Itinerary Note の）`title` が変化 |

比較キー:

| 種別 | キー |
|---|---|
| Trip Note | `owner_type=trip`, `title` |
| Day Note | `owner_type=day`, `day_number`, `title` |
| Itinerary Note | `owner_type=itinerary`, `day_number`, `sort_order`, `itinerary_key.title` |

v3 export を `load_trip_export_from_file` で読むと Itinerary は flatten されますが、**Expense は diff 非対象**（v1.6.0 時点）です。

## 非対象（将来バージョン）

以下は schema v3 では含めません:

- Photo / Attachment
- Expense diff
- Participant / Settlement / Shared Expense
- Multi Currency conversion
- XML / XSD export
