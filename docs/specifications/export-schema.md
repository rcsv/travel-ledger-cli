# Export Schema

Caglla CLI の trip export / import JSON 形式。

## Schema versions

| `schema_version` | 状態 | 説明 |
|---|---|---|
| 未指定 | v1（effective） | 旧形式。`notes` なし |
| `1` | v1 | 明示的 v1。`notes` なし |
| `2` | v2（現行 export） | Note を含む |

Import 時の解釈:

- `schema_version` 未指定 → v1
- `schema_version: 1` → v1
- `schema_version: 2` → v2

v1 export は引き続き import 可能です。現行 CLI の `trip export` は `schema_version: 2` を出力します。

## Top-level structure (v2)

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

| フィールド | 必須 | 説明 |
|---|---|---|
| `schema_version` | export 時は付与 | 未指定 import は v1 扱い |
| `trip` | ✓ | Trip 本体（`start_date` / `end_date` 必須） |
| `itinerary_items` | ✓ | 日程一覧 |
| `checklist_items` | v1 では省略可 | チェックリスト |
| `notes` | v2 では付与 | Note 一覧（空配列可） |

`trip` / `itinerary_items` / `checklist_items` の v1 フィールド名は変更していません（`days` / `itineraries` / `checklists` にはしていない）。

## Notes (schema v2)

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

## validate-export (v1.4.0+)

`trip validate-export` の追加検証:

- `schema_version` がサポート範囲（未指定 / 1 / 2）
- `notes` が配列（キーがある場合）
- `owner_type` が有効値
- Day Note の `day_number` が旅行期間内
- Itinerary Note の `itinerary_key` 存在
- `itinerary_key` が export 内 `itinerary_items` に解決可能
- `title` / `body` の型（JSON 構造）

解決不能な Note は validation error です。

## 非対象（将来バージョン）

以下は schema v2 では含めません:

- Photo
- Expense
- Itinerary Checklist
- Shared Expense
- Multi Currency
