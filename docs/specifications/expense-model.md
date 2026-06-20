# Expense モデル（設計草案）

Caglla CLI / 将来 Web 版に向けた **Expense（支出）** エンティティの仕様メモです。  
**v1.5.0: DB / CLI CRUD 実装済み。v1.6.0: Export schema v3（nested export/import/validate）実装済み。**

> **実装後レビュー:** v1.22.0 での責務定義（Transaction Record Layer、Budget / Estimate 分離）は [expense-post-implementation-review.md](expense-post-implementation-review.md)。**Estimate（Planned Money）** の責務整理は [estimate-model.md](estimate-model.md)。本書（v1.5.0）は設計履歴として残す。

関連: [Itinerary モデル](itinerary-model.md) / [Estimate モデル](estimate-model.md) / [Day モデル](day-model.md) / [Note モデル](note-model.md) / [Export Schema](export-schema.md)

---

## 1. Expense の責務

Expense は、旅行中に発生した **金銭的支出の記録** を担います。

| 責務 | 説明 |
|---|---|
| **記録** | いつ・どの予定（Itinerary）に関連する支出か、いくら・何通貨かを残す |
| **参照** | 後から Trip / Day / Itinerary の文脈で支出を辿れる |
| **バックアップ** | export / import により、将来 Web 版へ移行可能な形で保持する |

Expense は **精算・割り勘・為替換算** までは v1.x では行いません。  
あくまで「支出ログ」であり、Settlement は将来フェーズの責務です。

### 既存モデルとの関係

```text
Trip
 ├─ Note / Photo / Checklist        ← 現行または設計済
 └─ Day
      └─ Itinerary
           ├─ Note                  ← v1.3.x 実装済
           ├─ Expense               ← 本仕様（Itinerary 配下のみ）
           └─ itinerary_items.note  ← 短いメモ列（併存）
```

| 項目 | `itinerary_items.note` | Note エンティティ | Expense |
|---|---|---|---|
| 目的 | 予定に付随する短いメモ | 自由記述の記録 | **金額付き支出** |
| 複数件 | 1 予定 1 フィールド | 可 | 可 |
| 必須情報 | なし | `body` | **`amount` + `currency`** |

**方針:** Expense は Note と **別テーブル・別 CLI** とする。金額・通貨・支払者など、ドメインが異なるためポリモーフィック `owner_type` パターン（Note 型）には載せない。

---

## 2. Itinerary 配下とする理由

Expense の親は **Itinerary のみ** とします。

| 階層 | Expense の有無 |
|---|---|
| Trip 直下 | **なし**（v1.x） |
| Day 直下 | **なし**（v1.x） |
| Itinerary 配下 | **あり**（`expenses.itinerary_id` → `itinerary_items.id`） |

Trip や Day に紐づく支出は、必ず **いずれかの Itinerary** にぶら下げます。Itinerary は「旅行中の行動を表す単位」であり、支出の文脈を保持するアンカーです（[Itinerary モデル](itinerary-model.md)）。

### 理由

| 観点 | 説明 |
|---|---|
| **文脈** | ガソリンスタンド、高速道路、レストラン、入場券など、支出の多くは「その予定を実行した結果」として発生する |
| **入力容易性** | 旅行中は Itinerary を起点に「この予定で ○○ 円」と記録する方が自然 |
| **Day との関係** | Day は `itinerary.day_id` / `day_number` 経由で間接的に辿れる。Day 直下 Expense は二重管理になりやすい |
| **将来拡張** | Shared Expense / Beneficiary は Itinerary（またはその Participant 集合）をアンカーに拡張しやすい |
| **Export** | Note と同様、`itinerary_key` による安定参照が確立済み |

### 運用イメージ

```text
Day 2
 └─ Itinerary「美ら海水族館」     → 入場料 Expense
 └─ Itinerary「国道58号 給油」   → ガソリン Expense
 └─ Itinerary「道の駅 昼食」     → 食事 Expense（title 未入力でも amount のみ可）
```

Trip 全体の交通費・宿泊費など **Itinerary に紐づけにくい支出** は、v1.x では **ユーザーが明示的に作成した Itinerary**（例: 「その他経費」「Trip 共通」）に載せる運用を許容する。  
**ダミー Itinerary の自動作成は行わない**（`trip add` 時などに生成しない）。必要な場合はユーザーが `itinerary add` で作成する。  
Trip 直下 Expense の追加は、Participant / Settlement 設計が固まるまで **見送り** する。

---

## 3. v1.x 最小モデル

### エンティティ関係

```text
Trip
 └─ Day
      └─ Itinerary (itinerary_items)
           └─ Expense (expenses.itinerary_id → itinerary_items.id)
```

### 第一候補: `expenses` テーブル

```sql
CREATE TABLE expenses (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    itinerary_id    INTEGER NOT NULL,
    title           TEXT,
    amount          INTEGER NOT NULL,
    currency        TEXT NOT NULL,
    paid_by_name    TEXT,
    expense_date    TEXT,
    note            TEXT,
    sort_order      INTEGER NOT NULL DEFAULT 0,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_expenses_itinerary
    ON expenses(itinerary_id);
```

### カラム定義

| カラム | 必須 | 型（案） | 説明 |
|---|---|---|---|
| `id` | ✓ | INTEGER PK | AUTOINCREMENT |
| `itinerary_id` | ✓ | INTEGER | 親 Itinerary（`itinerary_items.id`） |
| `title` | — | TEXT NULL | 店名・項目名。**省略可**（後から追記） |
| `amount` | ✓ | INTEGER | **最小通貨単位**の整数（後述） |
| `currency` | ✓ | TEXT | ISO 4217（例: `JPY`, `USD`）。v1.x は **換算なし** |
| `paid_by_name` | — | TEXT NULL | 支払者名（自由文字列）。将来 `participant_id` へ移行 |
| `expense_date` | — | TEXT NULL | 支出日 `YYYY-MM-DD`。省略時は Itinerary の Day から導出可能 |
| `note` | — | TEXT NULL | 補足メモ。**省略可** |
| `sort_order` | ✓ | INTEGER | 同一 Itinerary 内の並び（Note / Checklist と同型） |
| `created_at` / `updated_at` | ✓ | TEXT | `YYYY-MM-DD HH:MM:SS`（既存エンティティと同一） |

### 入力容易性（title / note の NULL 許可）

旅行中の典型パターン:

| パターン | 入力例 | 保存イメージ |
|---|---|---|
| 金額だけ | `--amount 1500 --currency JPY` | `title=NULL`, `note=NULL` |
| 店名だけ | `--amount 980 --currency JPY --title コンビニ` | `note=NULL` |
| 後から説明 | `expense update 3 --note お土産"` | `note` を後追い |

**必須とするのは `amount` + `currency` のみ。**  
これらが Expense の本質的情報であり、空の支出行は意味を持たない。

### amount の表現（v1.x）

| 方針 | 内容 |
|---|---|
| **DB** | **最小通貨単位の INTEGER**（JPY=円、USD=セント、EUR=セント）。REAL 列は採用しない |
| **CLI `--amount`** | **小数入力を許可**。通貨に応じて CLI 側で最小単位へ変換してから DB に保存 |
| **パース** | **浮動小数点型（`f64`）に頼らず、文字列パース** で扱う（誤差・端数の制御） |
| v1.x 非対象 | 為替レート・換算後金額・端数処理ルール |

#### CLI 入力 → DB 保存の例

| 入力 | currency | DB `amount` |
|---|---|---|
| `--amount 1500` | `JPY` | `1500`（円） |
| `--amount 12.50` | `USD` | `1250`（セント） |
| `--amount 12.5` | `USD` | `1250` |

実装イメージ:

- `--amount` は文字列として受け取り、通貨ごとの **小数桁数**（JPY=0、USD=2 等）に基づき整数へ変換
- 変換ロジックは `parse_amount_for_currency(input: &str, currency: &str) -> Result<i64>` のように **単一関数へ集約**（amount セクションと export/import 将来実装で共用）
- IEEE 754 への一度も変換しない方針を優先（必要なら十進文字列ライブラリまたは自前桁処理）

Settlement フェーズで `amount_minor` 命名へ揃える余地は残すが、初版から整数で統一する。

### currency（v1.x）

- 1 Expense 行 = **1 通貨**
- Trip 全体のデフォルト通貨・表示通貨は **将来**
- **v1.x では形式検証のみ**。将来 ISO 4217 実データ検証へ **内部実装を差し替え可能** な構造とする

#### 検証方針（確定）

| 項目 | v1.x の扱い |
|---|---|
| 正規化 | 入力を **大文字** へ（例: `jpy` → `JPY`） |
| 形式 | **3 文字**であること |
| 未知コード | **許可する**（`XXX` 等も形式が合えば通す） |
| ISO 4217 一覧 | **実装しない**（コード表・DB・Repository なし） |
| 抽象化 | Validator trait 等は **不要** |

実装時は検証を **単一関数** に集約する:

```rust
/// 通貨コードの形式を検証し、正規化した 3 文字コードを返す。
/// v1.x: 大文字化 + 3 文字チェックのみ。
/// 将来: 本関数の内部を ISO 4217 検証に差し替え可能。
fn validate_currency_code(code: &str) -> Result<String>;
```

目的は **将来のフックポイント確保** であり、現時点での高度な通貨管理機能の導入ではない。CLI・DB 保存・将来 export 検証はすべてこの関数（または同モジュール）経由とする。

### expense_date

| 状態 | 扱い |
|---|---|
| 指定あり | その日付を保存 |
| 省略 | 表示・集計時に `trip.start_date + (itinerary.day - 1)` を **導出**（Day モデルと同思想） |
| DB | NULL 許可（導出可能なら必須にしない） |

---

## 4. paid_by 設計と participant_id への移行

### v1.x: `paid_by_name`（文字列）

| 項目 | 方針 |
|---|---|
| 型 | `TEXT NULL` |
| 意味 | 「誰が立て替えたか」の **表示用ラベル** |
| 必須 | **任意**（未入力 = 不明 / 後で記入） |
| 例 | `"太郎"`, `"John"`, `"現金共用"` |

Participant テーブルが無い v1.x では、精算ロジックに使わず **記録・表示・export のみ** に留める。

### 将来: `participant_id`

```text
participants
 ├─ id
 ├─ trip_id
 ├─ display_name
 └─ ...

expenses
 ├─ paid_by_name        ← 移行期間は残す（fallback 表示）
 └─ paid_by_participant_id  ← NULL 可 FK → participants.id
```

### 移行方針（段階的）

| Phase | DB | 動作 |
|---|---|---|
| **v1.x** | `paid_by_name` のみ | 文字列で記録 |
| **v2.x（Participant 導入）** | `paid_by_participant_id` 追加（NULL 可） | CLI `--paid-by` は Participant 名解決 + ID 保存。未登録名は `paid_by_name` のみ |
| **移行スクリプト** | Trip 単位で `paid_by_name` を `participants` にユニーク作成し ID を backfill（任意・手動実行） |
| **安定後** | `paid_by_name` は **denormalized cache** として維持するか、Participant 削除時の表示 fallback に限定 |

**確定（初版 DDL）:**

- **`paid_by_participant_id` 列は含めない**
- v1.x は **`paid_by_name` のみ**
- Participants 導入時に **`paid_by_participant_id INTEGER NULL` 列を追加**（その時点で FK / 解決ロジックを設計）

### Beneficiary / Shared Expense との境界

| 概念 | v1.x | 将来 |
|---|---|---|
| 誰が払った | `paid_by_name` | `paid_by_participant_id` |
| 誰の分か | **未モデル化**（全員均等などは記録しない） | `expense_beneficiaries` 中間テーブル |
| 割り勘 | 非対象 | Shared Expense + Settlement |

v3 責務整理: [shared-expense-model.md](shared-expense-model.md)（Issue #30 Responsibilities Review）

---

## 5. 将来拡張モデル（整理のみ）

v1.x / v1.5.0 では **実装しない**。Export schema や DB 分割の参考として記載する。

```text
Trip
 ├─ participants[]
 └─ Day
      └─ Itinerary
           └─ Expense
                ├─ paid_by_participant_id
                └─ expense_beneficiaries[]  → participant_id, share_ratio | share_amount
```

| 将来エンティティ | 責務 |
|---|---|
| **Participant** | Trip 参加者。Expense の支払者・受益者の正規参照 |
| **Expense Beneficiary** | 支出の恩恵を受ける参加者と按分 |
| **Shared Expense** | 複数 Expense / Beneficiary を束ねた精算単位（抽象化） |
| **Settlement** | Trip 終了時の精算結果（誰が誰にいくら払うか） |
| **Exchange Rate** | 日付・通貨ペアごとのレート履歴 |
| **Multi Currency Conversion** | 表示通貨への換算（集計・レポート用） |

### 拡張時の原則

- v1.x の **Itinerary 配下** 原則は維持
- v1.x Expense 行は **そのまま読み込める**（新列は NULL デフォルト）
- 精算ロジックは **CLI サブコマンドまたは Web** に分離し、Expense CRUD とは独立

---

## 6. 外部キー / cascade 方針

Note モデル（案 C）と同型: **FK は張らず、アプリ側で cascade** を推奨。

| トリガー | Expense の扱い |
|---|---|
| `trip delete` | 当該 Trip 配下 Itinerary に紐づく Expense を **すべて削除** |
| `itinerary delete` | 当該 `itinerary_id` の Expense を削除 |
| `day swap` | Expense は Itinerary に紐づくため **Itinerary と一緒に Day 間を移動**（Expense 行自体は変更不要） |
| `trip update`（期間短縮で Day 削除） | Itinerary 削除に伴い Expense も削除 |

実装イメージ: `delete_expenses_for_itinerary` / `delete_expenses_for_trip` + `itinerary delete` / `trip delete` からトランザクション内で呼び出し（v1.3.1 の Note cascade と同型）。

create 時: `itinerary_id` が存在し、指定 Trip に属することを検証。

---

## 7. CLI 設計案（v1.5.x 実装想定）

### コマンド例

```bash
# 最小入力（amount + currency 必須）
expense add --itinerary 12 --amount 1500 --currency JPY
expense add --itinerary 12 --amount 12.50 --currency USD

# 店名・支払者・メモは任意
expense add --itinerary 12 --amount 980 --currency JPY \
  --title コンビニ --paid-by-name 太郎 --note 飲み物

expense list --itinerary 12
expense list --trip 1              # v1 から提供: Trip 配下を集約表示
expense show 1
expense update 1 --title 昼食 --note 後から追記
expense delete 1
```

### owner 指定（確定）

| 操作 | owner 指定 |
|---|---|
| `add` | **`--itinerary` 必須**（Itinerary 起点を維持） |
| `update` / `delete` | **Expense ID** で指定（Itinerary 起点の更新・削除） |
| `list` | **`--itinerary` または `--trip` のいずれか 1 つ必須** |
| `list --trip` | 当該 Trip 配下 **すべて** の Expense を集約表示（Itinerary 経由）。**v1 から提供** |
| `list --itinerary` | 当該 Itinerary 配下のみ |

Trip / Day 直下への `add` は v1.x では **不可**。

### その他 CLI 論点

| 論点 | 方針 |
|---|---|
| `--amount` | 必須。**小数可**（§3 文字列パース → INTEGER 変換） |
| `--currency` | 必須。`validate_currency_code()` 経由 |
| `--title` / `--note` | 省略可 |
| `--paid-by-name` | 省略可 |
| `--expense-date` | 省略可（省略時 NULL、表示は Day から導出） |
| `--json` | `list` / `show` で対応（Note と同型） |

---

## 8. Export / Import（schema v3 — 実装済み）

現行 export は [Export Schema v3](export-schema.md)（`days[].itineraries[].expenses[]`）。  
top-level `expenses[]` 配列は **採用していません**。親子構造で Itinerary–Expense 関係を保持します。Itinerary の意味論は [Itinerary モデル](itinerary-model.md)、JSON 構造・検証は [Export Schema](export-schema.md) を参照してください。

### schema v3（実装）

```json
{
  "schema_version": 3,
  "trip": {},
  "days": [
    {
      "day_number": 2,
      "itineraries": [
        {
          "title": "美ら海水族館",
          "sort_order": 0,
          "start_time": "09:00",
          "expenses": [
            {
              "title": null,
              "amount": 2200,
              "currency": "JPY",
              "paid_by_name": "太郎",
              "expense_date": "2026-04-27",
              "note": null,
              "sort_order": 0
            }
          ]
        }
      ]
    }
  ],
  "checklist_items": [],
  "notes": []
}
```

| 論点 | 方針 |
|---|---|
| v2 との互換 | v2 export（Expense なし）は import 継続 |
| 内部 ID | **`expenses.id` / `itinerary_id` は export しない** |
| Itinerary 参照 | JSON 親子構造（`itinerary_key` は使わない） |
| import 順序 | Trip → Itinerary → Checklist → Note → **Expense** |
| `validate-export` | nested `expenses[]` の `currency` 必須・形式、`expense_date` 形式 |
| `trip duplicate` | v3 export/import 経由で Expense も複製（v1.7.0+） |
| `export-md` | Itinerary 下に簡素一覧（データ確認用、v1.7.0+） |
| `trip stats` | 件数・通貨別合計（換算なし、v1.7.0+） |

### リリース分割

| フェーズ | 内容 |
|---|---|
| **v1.5.0** | `expenses` テーブル + CLI CRUD + cascade + テスト |
| **v1.6.0** | Export schema v3 + import + validate-export |
| **v1.7.x** | duplicate / roundtrip 安定化、export-md / stats、canonical sample |
| **v1.9.x 以降** | trip diff Expense、安定比較キー |

---

## 9. Diff への影響

`trip diff` は v1.4.1 で `notes[]` に対応済み。Expense は **schema v3 以降** で `expenses[]` を比較対象に追加する。

### 比較キー（案）

同一 Itinerary に複数 Expense があるため、**`itinerary_key` + `sort_order`** を第一キーとする（Note の Itinerary キー + 同一 owner 内 sort と同型）。

| 操作 | 表示例 |
|---|---|
| 追加 | `+ Expense added: Itinerary / Day 2 / 美ら海水族館 / ¥2,200` |
| 削除 | `- Expense removed: ...` |
| 変更 | `~ Expense changed: ...`（`amount`, `currency`, `title`, `note`, `paid_by_name`, `expense_date` の変化） |

| 論点 | 方針 |
|---|---|
| v2 vs v3 比較 | v2 側 `expenses` 省略 = 空配列。**panic しない**（Note と同型） |
| キー変更 | `sort_order` 変更は remove+add に近い挙動になりうる — 将来 `expense stable key`（UUID）は **v1.x 非対象** |

---

## 10. Migration 戦略

### DB migration（初回追加）

```text
既存 DB (v1.4.x)
  → ALTER なし
  → 新規 `expenses` テーブル CREATE のみ
  → backfill 不要
```

| 項目 | 方針 |
|---|---|
| 既存 Trip | Expense 0 件のまま |
| `db reset` | 開発用。本番相当 DB は migration スクリプトで `expenses` 追加 |
| ロールバック | テーブル DROP のみ（Expense 未使用 DB では安全） |

### `paid_by_name` → `participant_id` migration（将来）

1. `participants` テーブル追加
2. `expenses.paid_by_participant_id` 列追加（NULL デフォルト）
3. オプション: Trip ごとに distinct `paid_by_name` から Participant 自動生成
4. CLI は Participant 優先、fallback で `paid_by_name` 表示
5. **旧 export（v3）** は `paid_by_name` のみ — import 時に Participant 未解決なら名前のみ復元

### Export schema migration

| From | To | 互換 |
|---|---|---|
| v2 | v3 | v2 import 継続。v3 export に `expenses[]` 追加 |
| v3 | v4（Participant 等） | `expenses[]` 内に optional `paid_by_participant_ref` 追加を想定 |

---

## 11. Doctor / Stats との関係（将来）

| 機能 | v1.5.x 想定 |
|---|---|
| `trip doctor` | **原則対象外**（Expense 未入力は warning にしない） |
| `trip stats` | 将来: Trip 合計支出・通貨別サマリー（換算なし集計から） |
| `trip advisor` | 対象外 |

将来例: 「Itinerary に category=restaurant があるが Expense が 0」→ optional suggestion。

---

## 12. あえて今回（v1.x / v1.5.0）やらないこと

以下は仕様に **名前だけ残し、実装・Export・精算ロジックは行わない**。

| 非対象 | 理由 |
|---|---|
| **Participants** | `paid_by_name` で足りる。正規化は精算フェーズ |
| **Expense Beneficiaries** | 按分・割り勘の前提設計が必要 |
| **Shared Expense** | 複数 Expense の抽象化。v1.x は 1 行 = 1 支出 |
| **Settlement** | 誰が誰に払うかの計算。CLI スコープ外 |
| **Multi Currency Conversion** | レート・表示通貨・端数 |
| **Exchange Rate History** | 換算の前提データ |
| Trip / Day 直下 Expense | Itinerary アンカーを優先 |
| Trip デフォルト通貨 | 全 Expense が同一通貨とは限らない |
| レシート Photo 添付 | Photo モデル未設計 |
| `trip diff` Expense | Export v3 実装後（v1.6.x 以降） |
| Export / Import / validate-export | **v1.5.x 非対象**（v1.6.x 以降） |
| **XML export / import** | v1.x 非対象（§17 参照） |
| **XSD 本格定義** | v1.x 非対象 |
| 精算レポート export | Settlement 実装後 |

---

## 13. 推奨案サマリー

| 項目 | 推奨 |
|---|---|
| 親 | **Itinerary のみ**（`itinerary_id`） |
| テーブル | 単一 `expenses` |
| 外部キー | **FK なし + 手動 cascade**（Note と同型） |
| 必須 | `amount` + `currency` |
| 任意 | `title`, `note`, `paid_by_name`, `expense_date` |
| amount | **INTEGER・最小通貨単位**。CLI は **文字列パース** で小数入力可 |
| currency | `validate_currency_code()` — **形式検証のみ**、未知コード許可 |
| paid_by | v1.x は **`paid_by_name` のみ**（`paid_by_participant_id` 列なし） |
| sort_order | **あり**（同一 Itinerary 内の複数 Expense 用） |
| CLI list | **`--itinerary` または `--trip`**（Trip 集約 list は v1 から） |
| Export | **schema v3** — **v1.6.x 以降**（v1.5.x は CRUD のみ） |
| Diff | schema v3 + `itinerary_key` + `sort_order`（v1.6.x 以降） |
| 正規 exchange 形式 | **JSON export schema**（XML は将来候補のみ） |

---

## 14. 実装フェーズ（確定）

| Phase | 内容 | リリース目安 |
|---|---|---|
| 1 | 本仕様メモ + レビュー反映 | **v1.5.0 設計** |
| 2 | DB / Model / migration | v1.5.x |
| 3 | CLI CRUD + `--json` + `list --trip` | v1.5.x |
| 4 | cascade / トランザクション / テスト | v1.5.x |
| 5 | Export schema v3 + import + validate-export | **v1.6.x 以降** |
| 6 | trip diff / export-md / stats | 将来 |

---

## 15. 確定方針（Open Questions 反映）

仕様レビューで確定した論点:

| # | 論点 | **確定方針** |
|---|---|---|
| 1 | `expense list --trip` | **v1 から提供**。`add` / `update` / `delete` は Itinerary 起点を維持。`list` のみ Trip 集約を許可 |
| 2 | `--amount` CLI 入力 | **小数入力を許可**。DB は INTEGER のまま。JPY `1500`、USD `12.50` → `1250`。**文字列パース**（浮動小数点非依存） |
| 3 | `currency` | 大文字 3 文字を基本。**未知コードは許可**。`validate_currency_code()` に集約。v1.x は形式検証のみ。将来 ISO 4217 は **関数内部差し替え** |
| 4 | ダミー Itinerary「その他経費」 | **自動作成しない**。運用例として仕様書に残す。必要時はユーザーが `itinerary add` |
| 5 | schema v3 と CRUD | **分離**。v1.5.x = CRUD のみ。Export / Import / validate-export = **v1.6.x 以降** |
| 6 | `paid_by_participant_id` | **初版 DDL に含めない**。Participants 導入時に NULL 可列を追加 |

---

## 16. Future considerations — XML / XSD

v1.x の **正規 export 形式は JSON export schema**（現行 v2、Expense 追加時 v3）とする。

**XML / XSD** は、将来の **データ交換形式・仕様表現の候補** として位置づける:

- 他システム連携、会計ソフト、レガシー import 等で XML が必要になった場合の選択肢
- JSON schema を XSD へ機械変換する、または並行して XSD を保守する、等は **未決定**

**v1.x で実装しないもの:**

- XML による `trip export` / `trip import`
- XSD による本格的スキーマ定義・検証パイプライン
- XML と JSON の双方向同期

JSON export schema が一次仕様である限り、Expense を含む全エンティティの **真のソース・オブ・トゥルース** は `docs/specifications/export-schema.md`（および v3 拡張）とする。

---

## 17. 実装に進む場合のリスク

| リスク | 緩和 |
|---|---|
| Note / Expense の UX 混同 | CLI help・README で「メモ vs 金額」を明示 |
| 1 Itinerary に多数 Expense | `sort_order` + list 表示の上限は設けない（v1.x） |
| 通貨混在 Trip の集計 | v1.x は換算しない。stats は通貨別内訳のみ |
| export キー衝突 | `itinerary_key` 解決失敗は import error（Note 実績を流用） |
| 精算期待の先走り | 本仕様・Release Notes で Settlement 非対象を明記 |
