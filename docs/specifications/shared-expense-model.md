# Shared Expense Model Responsibilities Review

Caglla.Travel CLI / 将来 Web 版に向けた **Shared Expense（v3）** — Expense と Participant の関係、支払者・負担者、精算の **責務整理** です。

**v3.0.0 設計フェーズ 1/6: Responsibilities Review のみ。** 本書は Entity Design・実装・export schema 変更を伴わない。フィールド詳細・DDL・CLI オプション名は Issue #31 以降。

| ドキュメント | 役割 |
|---|---|
| **本書** | Shared Expense の責務・境界・v3 スコープ |
| [shared-expense-entity-design.md](shared-expense-entity-design.md) (#31) | テーブル・フィールド・export v5（Entity Design） |
| [shared-expense-implementation-plan.md](shared-expense-implementation-plan.md) (#32) | 実装計画（Implementation Plan） |
| [shared-expense-post-implementation-review.md](shared-expense-post-implementation-review.md) (#34) | 実装後レビュー・Release 判定 |
| [participant-model.md](participant-model.md) (v2.0.0) | Participant = Trip 参加行の正本。v3 の参照先 |
| [expense-model.md](expense-model.md) (v1.5.0) | Expense = Transaction Record Layer |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) (v1.22.0) | v1 実装後の Expense 責務・v3 引き継ぎ |
| [planning-design-principles.md](planning-design-principles.md) (v2.0.1) | 入力過多を避ける判断軸 |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | 製品 v3 ロードマップ |

関連: [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) / [foundation-hardening-review.md](foundation-hardening-review.md) / [export-schema.md](export-schema.md) / [github-workflow.md](../github-workflow.md)

設計系列（Epic #13）:

```text
#30 Responsibilities Review   → shared-expense-model.md（本書）
#31 Entity Design             → shared-expense-entity-design.md
#32 Implementation Plan       → shared-expense-implementation-plan.md
#33 Implementation            → DB + CLI + export v5（想定）
#34 Post-Implementation Review → shared-expense-post-implementation-review.md
#35 Release v3.0.0
```

---

## Purpose

v3 **Shared Expense** の入口として、Expense が **何を表すか**、Participant と **どこまで結び付けるか**、**何を持たないか** を定義する。

```text
誰が払ったか・誰の費用か — グループ旅行の支出を、必要なときだけ構造化して記録する。
```

v3 は **Transaction Record の拡張** が目的であり、会計ソフト化・全自動仕訳・複雑な按分は **意図的に範囲外** とする。

> **Epic #13 補足方針（着手前合意）:**
>
> - v3.0.0 は **shared expense recording** を **settlement automation より優先** する
> - 既存 `expense add` は Participant / payer / split **なしでも従来どおり使える**
> - Shared Expense の詳細指定は **opt-in**
> - export schema **v5** は責務・エンティティ境界が固まってから検討（本書では方針のみ）
> - Settlement は **シンプルな範囲に限り** v3 に含めうるが、Caglla を会計アプリにしない

---

## Background

### v2 完了時点

v2.0.0 Participant Foundation + v2.0.1 hardening により、次が揃っている。

```text
Trip
 ├─ participants[]     ← export schema v4 正本（Trip 直下）
 └─ Day → Itinerary → Expense
                        └─ paid_by_name のみ（文字列ラベル）
```

| 成果 | v3 への意味 |
|---|---|
| `participants` CRUD + `is_self` | payer / beneficiary の **参照先 ID** が存在する |
| export v4 `participants[]` | import 順序: Participant → Expense の前提が整う |
| Expense = Transaction Record | [expense-post-implementation-review.md](expense-post-implementation-review.md) の v1 判定を **維持** |

canonical sample（`okinawa_sesoko_2026`）では `paid_by_name` に「Alex」「Jordan」等が記録されている。**Participant 行との自動リンクは v2 では行わない**。

### v1 / v2 での「誰が払ったか」の限界

| 現状 | 限界 |
|---|---|
| `paid_by_name` | 自由文字列。Participant との正規参照がない |
| beneficiary | **未モデル化** — 全員均等・個人負担の区別を記録できない |
| Settlement | **未実装** — 誰が誰にいくら払うかは計算・表示ともにない |

### v3 の位置づけ

[long-term-version-strategy.md](../long-term-version-strategy.md) §v3:

```text
ここで初めて Expense が Participant と結び付く。
Paid By / Beneficiary /（必要なら）Settlement
```

[participant-model.md](participant-model.md) §Deferred Scope と整合する。v2 Participant ID が **payer / beneficiary 解決の正本** である。

---

## Conceptual model: Expense remains Transaction Record

### テーマ 1 — Expense = Transaction Record のまま拡張するか

**結論: はい。** v3 でも Expense の中心責務は変えない。

| 観点 | 方針 |
|---|---|
| **Expense が表すもの** | 旅行中に **実際に支払った金額** の 1 取引（Transaction Record） |
| **Shared Expense** | **別エンティティの正本ではない** — 1 行の Expense に payer / beneficiaries メタデータを載せる **パターン** |
| **Settlement** | Expense 行そのもの **ではない** — Expense 群を **入力** とする **派生結果**（計算・表示レイヤー） |
| **Planned Money** | Budget / Estimate は **非対象**（Epic #13 Non-goals） |

```text
Expense（正本）
  amount, currency, title, note, itinerary_id
  + paid_by_participant_id   （optional — 誰が立て替えたか）
  + expense_beneficiaries[]  （optional — 誰の費用か / 均等按分）

Settlement（派生・v3.0.0 では最小）
  Expense + Participant を読んで「だいたい誰がいくら前払いしたか」を示す
  — 永続エンティティ・支払い消込は v3.0.0 スコープ外（§Settlement scope）
```

[expense-post-implementation-review.md](expense-post-implementation-review.md) §5 の 3 層（payer / beneficiary / settlement）を **v3 で初めて実装可能にする** が、Expense の **金額正本** は v1 から変わらない。

### Itinerary 配下・複数 Expense

[planning-design-principles.md](planning-design-principles.md) §5–6 を **維持** する。

| 関係 | 方針 |
|---|---|
| **1 Itinerary : N Expense** | **自然** — フードコート・複数店舗・追加注文など |
| **1 Itinerary : N Reservation** | 許容だが **行動分割のヒント** — Shared Expense とは独立 |
| **Shared Expense 粒度** | **Expense 行単位** — Itinerary を「精算バッチ」にしない |

Reservation の複数と Expense の複数を混同しない:

```text
Reservation 複数 → 行動分割を検討するヒントになりやすい
Expense 複数     → 同一行動内の複数取引として自然
Shared Expense   → 各 Expense 行に opt-in で payer / beneficiaries を付ける
```

---

## Payer semantics（`paid_by_participant_id`）

### テーマ 2 — 意味論

| 項目 | 方針 |
|---|---|
| **列** | `paid_by_participant_id` — `participants.id` への **optional FK**（NULL 可） |
| **意味** | この取引の **立替者 / 実際に支払いを実行した Participant** |
| **必須** | **任意** — 未指定 = 不明または個人旅行で区別不要 |
| **`paid_by_name`** | **維持** — denormalized 表示・Participant 未登録時の fallback・export 可読性 |
| **Trip 整合** | payer の Participant は **同一 Trip** に属すること（Entity Design で検証） |

### 解決と表示

| 状態 | 表示・意味 |
|---|---|
| `paid_by_participant_id` のみ | Participant の `name` を正とする |
| `paid_by_name` のみ | v1/v2 互換 — **文字列記録**（精算入力には使わない） |
| 両方あり | **ID を正**、`paid_by_name` は cache / export 用（不一致時の優先順位は #31） |
| 両方なし | payer **unknown** — 有効な Expense 行のまま |

### CLI 方針（責務レベル）

- 既存 `expense add --amount … --currency …` は **変更なし**（payer 省略可）
- payer 指定は **opt-in** — 例: `--paid-by-participant <id|name>`（具体名は #31）
- Participant 名解決は **同一 Trip 内** のみ

---

## Beneficiaries and personal vs shared

### テーマ 3 — 最小モデル

**結論: v3.0.0 は「均等按分のみ」の最小 beneficiary モデル** とする。加重按分・金額指定 split は **defer**。

| 概念 | v3.0.0 方針 |
|---|---|
| **personal expense** | beneficiary **未指定** — デフォルト。`paid_by_participant_id` があれば **その人の個人支出** と解釈 |
| **shared expense** | beneficiary を **1 名以上明示** — 列挙された Participant 間で **均等按分** |
| **全員で割り勘** | `--shared-with all` 相当の sugar — 実体は Trip の全 Participant を beneficiary に展開（#31 で CLI 設計） |
| **按分比率** | v3.0.0 **非対象** — `share_ratio` / `share_amount` は Entity Design で **defer 明記** |
| **中間テーブル** | `expense_beneficiaries`（`expense_id`, `participant_id`）— **均等 split のみ** |

```text
デフォルト（beneficiary なし）
  → personal / unknown split — 精算計算には含めないか、payer 個人として扱う

opt-in（beneficiary あり）
  → shared — amount を beneficiary 数で均等割り
```

### 「Shared Expense エンティティ」について

[expense-model.md](expense-model.md) §5 にあった **Shared Expense 抽象化**（複数 Expense を束ねる）は、v3.0.0 では **導入しない**。

| 案 | 判定 |
|---|---|
| 独立 `shared_expenses` テーブル | **v3.0.0 非採用** — 過剰抽象化 |
| Expense 行 + beneficiaries | **採用** — Transaction Record を拡張する最小形 |

将来、レストラン 1 会計を複数 Expense 行に分けた場合も、**beneficiary 集合が同じ Expense 群** として精算入力に使える（バッチ ID は不要）。

### テーマ 4 — participant 未登録 / self unknown

| 状態 | Expense の扱い |
|---|---|
| **participants 0 件** | `expense add` **従来どおり** — `paid_by_name` のみ可能。structured payer / beneficiary **不可**（Participant 参照がないため） |
| **participants あり、`is_self` 不明** | Expense 記録は **可能** — statistics の `participant_count` とは **独立**（[participant-model.md](participant-model.md) §Participant count semantics） |
| **payer = deleted participant** | 行は **保持** — 表示は fallback（`paid_by_name` または「削除済み参加者」ラベル）。Entity Design で FK / SET NULL 方針を確定 |
| **beneficiary = deleted participant** | beneficiary 行削除または dangling 表示 — #31 で確定 |

**原則:** Participant 未整備の Trip でも **v1/v2 同等の Expense 記録を壊さない**。Shared Expense 構造は **データが揃ったときだけ opt-in**。

---

## Settlement scope

### テーマ 5 — v3 に含めるか

**結論: v3.0.0 は recording 優先。Settlement は「読み取り専用の簡易集計」までを上限とし、精算コマンド・永続 Settlement エンティティ・支払い消込は defer。**

| 項目 | v3.0.0 | defer（v3.x / 以降） |
|---|---|---|
| payer / beneficiary **記録** | **含める** | — |
| `expense list` / export-md で payer・shared 表示 | **含める**（表示強化） | — |
| Trip 単位の **読み取り専用** 集計（前払い概算） | **含めてよい** — シンプルな合計・Participant 別立替額 | 複雑な換算 |
| `expense settlement` **計算コマンド** | **含めない** | v3.x 候補 |
| **誰が誰にいくら払うか** の transfer リスト | **含めない** | v3.x 候補 |
| Settlement **永続エンティティ** | **含めない** | 必要なら将来 |
| 支払い済みフラグ・消込 | **含めない** | 会計ソフト領域 |
| 多通貨換算を用いた精算 | **含めない** | Exchange Rate 系と連動 |

```text
v3.0.0 のユーザー価値:
  「誰が何を立て替えたか」「この会計は誰の分か」を Trip 内で構造化して残せる

v3.0.0 が目指さないもの:
  「Jordan → Alex ¥1,234 を PayPay で送金済」レベルの清算管理
```

[expense-post-implementation-review.md](expense-post-implementation-review.md) が defer していた Settlement を、**最小の read-only summary** までは v3 Epic 内で再検討してよいが、**Epic #13 補足方針** により automation は後ろに置く。

---

## Note vs structured fields

### テーマ 6 — 境界（planning-design-principles 準拠）

[planning-design-principles.md](planning-design-principles.md) §4:

```text
精算に必要な構造だけを増やし、雑多なメモまでフィールド化しない。
```

| 構造化する（v3） | Note / 自由記述に残す |
|---|---|
| 立替者（Participant 参照） | 「現金で割った」「レジで別会計」 |
| shared 対象（beneficiary 列挙） | 「Jordanが多め出した」（**非均等** — v3.0.0 では構造化不可） |
| 金額・通貨・店名 | レシート番号の長文、経緯、言い訳 |
| personal vs shared の **事実** | 割り勘の **交渉メモ** |

**判断ルール:**

1. 均等割りで表せる shared → **beneficiary 構造化**
2. 非均等・曖昧 → **Note**（または v3.x の weighted split まで defer）
3. 精算に必須でない背景 → **Expense `note` または Trip/Itinerary Note**

Checklist 担当者割当（`assigned_participant_id`）は Shared Expense とは **別テーマ** — [planning-design-principles.md](planning-design-principles.md) §3 どおり v3 以降の検討でよい。

---

## Responsibilities

Shared Expense（v3）が **担う** こと:

| 責務 | 説明 |
|---|---|
| **立替者の構造化** | optional `paid_by_participant_id` — Participant 正本への参照 |
| **負担者の構造化** | optional `expense_beneficiaries` — **均等按分** の shared expense |
| **personal デフォルト** | beneficiary 未指定時は shared 精算入力に **自動では載せない** |
| **opt-in UX** | 既存 `expense add` 最小入力パスを **維持** |
| **Trip スコープ整合** | payer / beneficiary は **同一 Trip** の Participant のみ |
| **表示・export** | payer / beneficiaries をバックアップ・しおりで **読める** |
| **v2 互換** | `paid_by_name` のみの Expense は **意味変更なし** |
| **Transaction Record 維持** | 金額正本は Expense 行 — Itinerary 配下のまま |

```text
Shared Expense answers: who paid? who shares this cost? (when explicitly recorded)
Not: full accounting, weighted splits, or payment settlement workflow.
```

---

## Non-responsibilities

Shared Expense / v3 Expense 拡張が **担わない** こと:

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **Person / Traveler Profile** | Root 正本 | 将来 Person（Epic #13 Non-goals） |
| **Budget / Estimate** | Planned Money | 将来 / 非対象 |
| **会計ソフト級仕訳** | 入力過多・会計アプリ化 | 外部ツール |
| **加重按分・金額指定 split** | v3.0.0 最小スコープ外 | v3.x Entity 拡張 |
| **独立 Shared Expense バッチ** | 過剰抽象化 | Expense 行 + beneficiaries |
| **永続 Settlement / 消込** | recording 優先 | v3.x 以降 |
| **多通貨精算換算** | Exchange Rate 未実装 | 将来 |
| **Reservation 名義人 FK** | 別ドメイン | Reservation 拡張（optional 将来） |
| **領収書画像・OCR** | メディア / 自動化 | 将来 |
| **Participant 権限** | ローカル CLI 前提 | User / Cloud（v7–v8） |

Epic #13 Non-goals を **そのまま v3 Responsibilities の Non-goals** とする。

---

## v3.0.0 Scope（責務上の約束）

設計系列の **Phase 1 完了時点** で確定するスコープ:

```text
✓ Expense は Transaction Record のまま
✓ optional paid_by_participant_id（Participant FK）
✓ optional expense_beneficiaries（均等 split のみ）
✓ personal デフォルト — shared は opt-in
✓ expense add 最小入力は v2.0.1 と同等に維持
✓ paid_by_name 共存 — Participant 未登録 Trip も記録可能
✓ payer / beneficiary の export・一覧表示
✓ 1 Itinerary : N Expense 維持

✗ 加重按分 / share_amount / share_ratio
✗ 独立 shared_expenses エンティティ
✗ 永続 Settlement テーブル
✗ expense settlement 計算 CLI（transfer リスト）
✗ 支払い済み・消込
✗ ISO 4217 換算付き精算
✗ v2.0.x DB / export v4 の破壊的変更
```

### 想定のユーザー価値

```text
グループ旅行で「この昼食は全員で割り」「この入場券はAlexの個人」と残せる。
Participant を登録していれば CLI / export で参照付き記録ができる。
Participant がなくても、今まで通り amount + paid_by_name だけで使える。
旅行後の「だいたい誰がいくら出したか」を目視確認しやすい（自動精算は v3.0.0 必須ではない）。
```

---

## Deferred Scope

### v3.x / post-v3.0.0

| 項目 | 内容 |
|---|---|
| **Settlement 計算** | `expense settlement` — 誰が誰にいくら払うか（transfer リスト） |
| **加重按分** | `share_ratio` / `share_amount` |
| **永続 Settlement** | 計算結果の保存・確定 |
| **read-only summary の高度化** | 多通貨・カテゴリ別集計 |
| **paid_by_name → Participant backfill** | Trip 単位マイグレーション CLI（任意） |

### 他製品バージョン

| バージョン | 関係 |
|---|---|
| **v5 Travel Book** | export-md に payer / shared セクション — 表示レイヤー |
| **v6 Travel Journal** | 支出エピソードと Participant の **関連付け** は可能だが Journal 本体 |
| **Person / Traveler Profile** | Participant が `person_id` を参照する将来 — Shared Expense の FK 先は **Trip participation のまま** |

---

## Relationship with existing entities

### Participant

| 関係 | 方針 |
|---|---|
| **参照方向** | Expense → Participant（payer, beneficiary）。Participant → Expense の逆 FK は **不要** |
| **正本** | Participant **name** は participants テーブル。Expense は ID で参照 |
| **削除** | Participant 削除時の Expense 扱い — #31（fallback 表示優先） |
| **is_self** | payer デフォルト推定には **v3.0.0 では使わない**（暗黙 payer = self は magic になる） |

### Expense

| 観点 | v2.0.1 | v3.0.0 |
|---|---|---|
| 金額正本 | Expense | **変更なし** |
| `paid_by_name` | 有効 | **維持** |
| `paid_by_participant_id` | なし | **追加** |
| beneficiaries | なし | **optional 中間テーブル** |
| CRUD コマンド | 既存 | **後方互換** + opt-in オプション |

### Reservation

**直接リンクなし** — [participant-model.md](participant-model.md) §Reservation 方針を維持。

予約の名義・確認番号は Reservation 正本。Shared Expense は **支払い・按分** のみ。

### Note

beneficiary に載せられない **非均等割り**・経緯は Note / Expense `note`。**構造化の逃げ場** として残す。

### Itinerary / Day / Trip

- Expense は **Itinerary 配下** のまま（[expense-model.md](expense-model.md) §5 拡張原則）
- `trip delete` / `itinerary delete` cascade は v1 同型 — beneficiaries も連鎖削除（#31）

---

## Export / Import Considerations

**本フェーズ（#30）では export schema を変更しない。** 方針のみ記録。

| 論点 | Responsibilities Review 時点の方針 |
|---|---|
| **schema バージョン** | **v5 を想定** — 責務確定後に Entity Design で final（Epic #13 補足） |
| **v4 互換** | v4 import **継続**。新フィールドは optional / デフォルトで v1/v2 意味 |
| **配置** | beneficiaries は **nested** `expenses[].beneficiaries[]` が自然（Itinerary 配下維持） |
| **Participant 参照** | export では **name + sort_order** または `participant_ref` — #31 で確定 |
| **`paid_by_name`** | export に **残す** — roundtrip 可読性 |
| **import 順序** | Trip → **participants** → … → expenses（v2 と同型） |

```json
{
  "schema_version": 5,
  "participants": [ "..." ],
  "days": [{
    "itineraries": [{
      "expenses": [{
        "amount": "980",
        "currency": "JPY",
        "paid_by_name": "Alex",
        "paid_by_participant_ref": "Alex",
        "beneficiaries": [ { "participant_ref": "Alex" }, { "participant_ref": "Jordan" } ]
      }]
    }]
  }]
}
```

上記は **構造イメージ** であり、フィールド名・ref 形式は Entity Design の成果物。

---

## CLI Considerations

**本フェーズでは CLI を実装しない。** 方針のたたき台:

```bash
# 従来どおり（変更なし）
expense add --itinerary 12 --amount 1500 --currency JPY

# opt-in: payer
expense add --itinerary 12 --amount 980 --currency JPY \
  --paid-by-participant Alex

# opt-in: shared（均等 — beneficiary 列挙）
expense add --itinerary 12 --amount 4000 --currency JPY \
  --paid-by-participant Alex \
  --beneficiary Alex --beneficiary Jordan
```

| 論点 | 方針 |
|---|---|
| **デフォルト** | beneficiary なし = **personal** |
| **shared sugar** | 全 Participant を beneficiary にする省略記法 — #31 |
| **`expense update`** | payer / beneficiaries の **追加・変更・クリア** が可能であること |
| **Settlement CLI** | v3.0.0 **実装しない** |
| **validation** | beneficiary に Trip 外 Participant を **拒否** |

---

## Compatibility Considerations

### 既存 DB（v2.0.x）

| 項目 | v3 導入時 |
|---|---|
| Migration | `paid_by_participant_id` 列追加 + `expense_beneficiaries` 新規テーブル（#33） |
| 既存行 | 新列 NULL — **意味は v2 と同一** |
| `paid_by_name` | **削除しない** |

### 既存 export

| From | To | 互換 |
|---|---|---|
| v4 export | v5 import | 新フィールド省略 = personal / payer unknown |
| v5 export | v4 import | **不可** — 想定どおり |
| v4 export | v4 import | **v3 実装まで継続** |

### canonical sample

`okinawa_sesoko_2026` への payer / beneficiary 例示は **#33 以降の任意タスク**。Responsibilities Review では必須としない。

---

## Open Questions

Entity Design（#31）へ引き継ぐ項目:

| # | 質問 |
|---|---|
| 1 | export の Participant 参照 — `display_name` のみか `participant_ref` か |
| 2 | `paid_by_name` と `paid_by_participant_id` **不一致** 時の validate / doctor 方針 |
| 3 | Participant **削除** 時 — payer FK SET NULL vs RESTRICT |
| 4 | beneficiary **0 名の shared フラグ** を独立持つか、beneficiary 行の有無のみで判定するか |
| 5 | `--beneficiary all` の **is_self 未設定 Trip** での挙動 |
| 6 | export schema **v5 確定** vs v4 拡張 without bump — 互換マトリクス |
| 7 | read-only **Trip 集計** を v3.0.0 に含めるか — コマンド名・出力形式 |
| 8 | `trip diff` — payer / beneficiary 変更の検出粒度（[foundation-hardening-review.md](foundation-hardening-review.md) §Maintenance） |
| 9 | doctor / advisor — shared なのに beneficiary 1 名のみ等の **warning** 要否 |

---

## Completion Criteria

本 Responsibilities Review（Issue #30）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `shared-expense-model.md` が存在する | 本書 |
| 2 | Responsibilities / Non-responsibilities が明確 | §Responsibilities, §Non-responsibilities |
| 3 | v3.0.0 scope と deferred scope が明確 | §v3.0.0 Scope, §Deferred Scope |
| 4 | Epic #13 テーマ 1–6 が決定されている | 各 §Conceptual model 〜 §Note |
| 5 | Settlement は recording 優先で境界確定 | §Settlement scope |
| 6 | Entity Design (#31) へ Open Questions を接続 | §Open Questions |
| 7 | Rust / DB / export 実装なし | 本フェーズ対象外 |
| 8 | `make check` PASS | PR CI |

---

## Next phase notes（Entity Design #31）

#31 では本書の **Open Questions** を閉じ、以下を確定する:

- DDL（`paid_by_participant_id`, `expense_beneficiaries`）
- export schema v5 フィールド一覧と v4 import 互換
- CLI オプション名・エラーメッセージ・cascade 詳細
- validate-export / doctor ルール

Implementation Plan（#32）→ Implementation（#33）→ Post-Implementation Review（#34）→ Release v3.0.0（#35）の順で進める。
