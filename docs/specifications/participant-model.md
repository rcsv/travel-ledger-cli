# Participant Model Responsibilities Review

Caglla.Travel CLI / 将来 Web 版に向けた **Participant（旅行参加者）** エンティティの責務整理です。

> **用語注意:** 日本語の「同行者」は日常会話で **自分以外** を指しがちだが、Caglla の **Participant** は英語の participant と同様、**自分を含むその Trip の旅行参加者全員** を指す。自分以外のみは **Companion**（§Participant count semantics）と呼び分ける。

**v2.0.0 設計フェーズ 1/6: Responsibilities Review のみ。** 本書は Entity Design・実装・export schema 変更を伴わない。フィールド詳細は Issue #8 以降。

| ドキュメント | 役割 |
|---|---|
| **本書** | Participant の責務・境界・v2 スコープ |
| [participant-entity-design.md](participant-entity-design.md) (#8) | テーブル・フィールド・検証（Entity Design） |
| [participant-implementation-plan.md](participant-implementation-plan.md) (#9) | 実装計画（Implementation Plan） |
| [participant-post-implementation-review.md](participant-post-implementation-review.md) (#11) | 実装後レビュー・Release 判定 |
| [expense-post-implementation-review.md](expense-post-implementation-review.md) (v1.22.0) | Expense = Transaction Record、v3 精算への引き継ぎ |
| [reservation-responsibilities-review.md](reservation-responsibilities-review.md) (v1.19.0) | Reservation と Participant の独立性 |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | 製品 v2 / v3 ロードマップ |

関連: [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) / [expense-model.md](expense-model.md) / [export-schema.md](export-schema.md) / [github-workflow.md](../github-workflow.md)

設計系列（Epic #6）:

```text
#7  Responsibilities Review   → participant-model.md（本書）
#8  Entity Design             → participant-entity-design.md
#8+ Person / Trip 境界補正     → 本書・entity-design 更新（#9 前）
#8+ Participant count 意味論   → 本書・entity-design 更新（#9 前）
#9  Implementation Plan        → participant-implementation-plan.md
#10 Implementation             → PR #24 (export v4)
#11 Post-Implementation Review → participant-post-implementation-review.md
#12 Release v2.0.0
```

---

## Purpose

v2 **Participant Foundation** の入口として、Participant が **何者か**、**どこまで責任を持つか**、**何を持たないか** を定義する。

```text
誰と旅行するか — Trip に同行者を登録・参照できる状態を作る。
```

v2 は **Trip 内の参加関係レジストリ**（Trip-scoped participation record）の確立が目的であり、精算・割り勘・Expense 構造変更は **v3** に送る。

> **設計補正（#8 後・#9 前）:** v2 の `participants` は **人間そのものの正本** ではなく、**ある Trip に参加している行** として扱う。Root スコープの **Person / Traveler Profile**（パスポート・生年月日・マイレージ等）は **将来候補** であり v2.0.0 では実装しない。詳細は §Conceptual model。
>
> **設計補正（count 意味論）:** Participant は **自分を含む** 旅行参加者全員。人数統計では `participant_count` と `companion_count` を混同しない。v2.0.0 では `is_self` 列で「この Trip における自分」をマークする。詳細は §Participant count semantics。

---

## Background

### v1 完了時点

v1 Planning Foundation では、旅行計画・実績記録の基盤が揃った。

```text
Trip → Day → Itinerary
  + Checklist, Note, Summary, Expense, Reservation, Remark
```

### v1 での「誰」表現の限界

| 現状 | 限界 |
|---|---|
| Expense `paid_by_name` | 自由文字列。同一人物の正規化・一覧・参照がない |
| Note / Summary 本文 | 参加者名を書けるが **構造化データではない** |
| Checklist | Trip 配下だが **同行者エンティティではない** |

canonical sample（`okinawa_sesoko_2026`）では `paid_by_name` に「Alex」「Jordan」等が記録されているが、**Participant としての正本は存在しない**。

### v2 の位置づけ

[long-term-version-strategy.md](../long-term-version-strategy.md) §v2:

```text
Participant を Trip に紐付けられる。
この段階では精算機能は持ち込まない。
```

[expense-post-implementation-review.md](expense-post-implementation-review.md) §5 は、v2 で Participant 前提を満たしたうえで v3 で `paid_by_participant_id` 等を導入する方針と整合する。

---

## Conceptual model: Person vs Trip participation

概念的に、**人物そのもの** と **Trip への参加** は別レイヤーである。

```text
Person / Traveler Profile（将来・Root スコープ）
  = 再利用可能な人物情報の正本
    生年月日、パスポート、マイレージ、連絡先、アレルギー等

Trip participation record（v2.0.0 = 現行 participants テーブル）
  = ある Trip にその人が参加しているという参加関係
    trip_id, name（表示名）, sort_order 等
```

### v2.0.0 での整理

| 概念 | v2.0.0 | 備考 |
|---|---|---|
| **Person / Traveler Profile** | **実装しない** | Root スコープ。候補名: Person, Traveler, TravelerProfile, Contact 等 — **未確定** |
| **Trip participation（Participant）** | **実装する** | Trip 配下の `participants` テーブル。概念的には **TripParticipant-like record** |

v2 の Participant は **「その Trip に誰が参加しているか」を Trip 内で識別・表示するための行** であり、パスポート番号やマイレージのような **Trip 横断で再利用したい属性の正本ではない**。

### 将来の Person / Traveler Profile が持ちうる情報（v2 では未実装）

| 情報 | 例 |
|---|---|
| legal name | 旅券記載名 |
| display name | 日常の呼び名 |
| date of birth | 生年月日 |
| passport number / expiry | パスポート |
| mileage program | マイレージ |
| contact info | 連絡先 |
| allergy / care notes | アレルギー・配慮事項 |
| emergency contact | 緊急連絡先 |

これらは **毎 Trip ごとに再入力したくない** 性質のデータであり、将来 Root に Person / Traveler Profile を導入し、Trip participation がそれを **参照** する構造が有力な拡張候補である。

### 将来の migration path（v2.0.0 では実装しない）

```text
persons（将来）
  id
  display_name
  ...（上記プロフィール属性）

participants（v2 テーブル名は維持想定）
  id
  trip_id
  person_id   nullable   ← 将来追加候補
  name        ← Trip 内表示名（person 未リンク時は standalone 名）
  sort_order
  ...
```

- v2.0.0 では `person_id` 列・`persons` テーブル・`trip_participants` への **rename は行わない**
- 将来、`participants` を `trip_participants` にリネームする案は **Open Question**（[participant-entity-design.md](participant-entity-design.md) §Open Questions）

### 「グローバル再利用しない」との関係

| 表現 | 意味 |
|---|---|
| **v2.0.0 では** | グローバル再利用可能な Person / Traveler Profile は **実装しない** |
| **将来** | Root-level Person / Traveler Profile を導入し、Trip participation が `person_id` で参照する構造は **有力な拡張候補** |

**「v2 ではやらない」≠「将来やらない」** と区別する。v2 は最小実装として Trip 内参加行のみを導入する。

---

## Participant count semantics

### 背景（Bubble / caglla.travel の教訓）

statistics 表示で、Participant を「自分以外の同行者」とみなすか「自分を含む参加者全員」とみなすかで人数が変わる。特に次の式は **危険** である。

```text
companion_count = count(participants) - 1   ← is_self なしでは使わない
```

- participants に **自分が含まれていない** 前提だと、一人旅で `count = 0` のとき **-1** になる
- participants に **自分が含まれている** 前提だと、家族旅行で自分を登録し忘れたとき **過小評価** になる

**`count(participants) - 1` は、Trip 内に `is_self = true` がちょうど 1 件あることが保証されない限り使わない。**

### 用語

| 用語 | 意味 |
|---|---|
| **Participant** | その Trip に **参加する人全員**。**自分を含む** |
| **Companion / fellow traveler** | **自分以外** の同行者（日本語の「同行者」に近い） |
| **`participant_count` / `traveler_count`** | 旅行参加人数 — **自分を含む** 全 Participant 数 |
| **`companion_count`** | 自分以外の同行者数 |
| **`registered_participant_count`** | `participants` テーブルに登録されている行数（意味が曖昧なときの内部指標） |

### 推奨する算出規則（v2.0.0）

| 状態 | `participant_count` / `traveler_count` | `companion_count` |
|---|---|---|
| participants **未登録**（0 行） | **unknown / not recorded**（0 人と表示しない） | **unknown / not recorded** |
| 登録あり、`is_self = true` が **1 件** | `count(participants)` | `count(participants) - 1` |
| 登録あり、`is_self = true` が **0 件** | **unknown**（自分がリストに含まれるか不明） | **unknown** |
| 登録あり、`is_self = true` が **2 件以上** | **invalid** — 登録エラー（§Validation） | — |

**一人旅の例:** 自分のみ登録（`is_self = true` 1 件）→ `participant_count = 1`, `companion_count = 0`。

**未登録の一人旅:** `participant_count` は **0 ではなく unknown**。ユーザーがまだ参加者を記録していないだけである。

### `is_self` の意味

| 項目 | 方針 |
|---|---|
| **意味** | この Trip における **自分**（計画者・記録者・端末の利用者）を示す |
| **別名候補** | `is_primary`, `is_owner`, `is_planner` — v2 では **`is_self`** で統一 |
| **User account との関係** | local-first CLI ではログイン不要。`is_self` は **Trip 内のローカルマーカー** であり User ID ではない |
| **Person / Traveler Profile との関係** | 将来 `person_id` が入っても、**Trip participation 側の属性として `is_self` は残る**（どの参加行が「この端末の自分」か） |

### 案 A vs 案 B（`is_self` を v2 に含めるか）

| 観点 | 案 A: v2 で `is_self` を入れる | 案 B: v2 では `is_self` を入れない |
|---|---|---|
| 一人旅 | `participant_count=1`, `companion_count=0` と表現可能 | participants 未登録と区別しにくい |
| `count - 1` | `is_self` 保証時のみ安全 | **常に危険** — statistics で誤用されやすい |
| `count(participants)` の意味 | 旅行人数（self 含む）として解釈可能 | **登録済み行数** に留まり、旅行人数ではない |
| local-first | User なしで Trip 内マーカーとして成立 | companion_count は v2 では算出不能 |
| memo / role との差 | **人数意味論に直結** — v2 必須に近い | 表示名だけでは統計が破綻しやすい |
| #8 との整合 | 列 1 つ追加。DDL・export・CLI を #9/#10 で拡張 | 当初 DDL と一致するが #9 前に意味論補正が必要 |

**判断: 案 A を採用 — v2.0.0 の `participants` に `is_self` を含める。**

理由: Participant の意味を安定させ、一人旅・家族旅行・statistics で破綻しにくい。memo / role と違い人数計算の意味論に関わる。将来 Person が入っても Trip participation 側に残せる。

案 B（`is_self` なし）で v2 を出すと、statistics / docs で「登録数」と「旅行人数」の混同を防ぐのが難しい。**Deferred にはしない。**

### #9 前に残す Open Questions（`is_self` 関連）

| # | 質問 |
|---|---|
| 1 | `trip add` 時に **default self participant** を自動作成するか（名前は `"Me"` / 空禁止なら要デフォルト名） |
| 2 | `participant add --self` と `participant update --self` の UX（既存 `is_self` があるときの切り替え） |
| 3 | `participant list` / 将来 statistics で unknown をどう表示するか（`—` / `not recorded`） |
| 4 | export v4 に `is_self` を含めるか — **含める想定**（#9 で確定） |

---

## Responsibilities

### 定義

```text
Participant (v2) is a Trip-scoped participation record for someone who takes part in that trip —
including the trip planner / recorder (self).
It identifies and displays them within the trip; it is not the canonical master record for the person themselves.
```

日本語:

```text
Participant（v2）= ある Trip に参加している人を、その Trip 内で識別・表示するための参加行
（概念的には TripParticipant-like record）
自分を含む旅行参加者全員が対象。自分以外のみは Companion と呼ぶ（§Participant count semantics）。
```

Participant は **人間そのものの正本（Person / Traveler Profile）ではない**。また **アプリ利用者（User）でも、精算単位（Settlement）でも、予約名義でもない**。v2 では **その Trip における参加関係・表示名** の正本である。

### 基本責務

| 責務 | 説明 |
|---|---|
| **参加者の登録** | Trip に 0..N 人の Participant を追加できる。**自分を含む** 全員が対象 |
| **表示名の保持** | しおり・一覧・将来 UI で使う **人を指すラベル** |
| **`is_self` マーカー** | その Trip における **自分** を最大 1 件までマーク（§Participant count semantics） |
| **Trip スコープ** | Participant は **特定 Trip にのみ属する参加行**。v2.0.0 では Root-level Person / Traveler Profile は **未実装**（§Conceptual model） |
| **並び順** | 同一 Trip 内での表示順（`sort_order` 想定 — 詳細は Entity Design） |
| **v3 の前提データ** | 将来 Expense の payer / beneficiary 解決の **参照先** となる ID を持つ |
| **バックアップ** | 将来 export v4 で Trip と一緒に移行可能な形で保持 |

### Trip との紐づけ

```text
Trip
 └─ Participant[]     ← v2 で追加（Trip 直下のみ）
```

| 階層 | v2 の Participant |
|---|---|
| **Trip** | **あり** — 正本の親 |
| **Day** | **なし** |
| **Itinerary** | **なし**（直接紐づけしない） |

Day / Itinerary は **いつ・何をするか** の正本。Participant は **誰が行くか** の正本。時系列・行動とは直交する。

### v2.0.0 で扱う最小情報（責務レベル）

Entity Design（#8）でフィールドを確定する。Responsibilities Review 時点の **最小イメージ**:

| 情報 | v2 で必要か | 備考 |
|---|---|---|
| **表示名**（`name`） | **必須** | しおり・CLI 一覧の主キー的ラベル |
| **`is_self`** | **v2 で導入** | この Trip における自分。同一 Trip で **最大 1 件**（§Participant count semantics） |
| **並び順** | **推奨** | Checklist / Note と同型 |
| **補足メモ** | 任意 | 年齢・関係など — 構造化は将来 |
| **User ID / メール** | **v2 では不要** | Identity（製品 v7）の領域 |
| **権限・ロール** | **v2 では不要** | 編集権限はローカル CLI 前提 |
| **パスポート・生年月日・マイレージ等** | **v2 では不要** | 将来 **Person / Traveler Profile**（Root）の領域 |

---

## Non-responsibilities

Participant が **担わない** こと:

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **精算・割り勘** | 計算ロジック | **Settlement**（v3） |
| **誰が払った / 誰の費用か** | Expense 構造拡張 | **v3 Shared Expense**（`paid_by_participant_id`, beneficiary） |
| **金額** | 金銭正本 | **Expense** |
| **予約・確認番号** | 手続き正本 | **Reservation** |
| **予約名義人** | v2 ではモデル化しない | 将来検討 — v2 では Reservation に Participant リンクなし |
| **ログイン・同期** | アカウント | **User / Cloud**（v7–v8） |
| **旅行記・写真** | Story / メディア | **Travel Journal**（v6） |
| **旅行全体の要旨** | Abstract | **Summary** |
| **自由記述メモ** | Annotation | **Note** |
| **準備項目** | チェック可能タスク | **Checklist** |
| **行動の時系列** | 旅程正本 | **Itinerary** |
| **パスポート・連絡先・マイレージ** | 人物マスター | **Person / Traveler Profile**（将来・Root） |

```text
Participant answers: who is on this trip? (as a participation record)
Not: the person's global profile, what they paid, what they booked, or what they wrote.
```

---

## Relationship with existing entities

### Trip

| 関係 | 方針 |
|---|---|
| **親** | Participant は **Trip にのみ** 属する |
| **削除** | `trip delete` で当該 Trip の Participant を **すべて削除**（cascade 想定） |
| **複製** | `trip duplicate` 時に Participant も複製するのが自然（#10 で設計） |
| **一覧** | `participant list --trip <id>` が主な参照経路 |

### Day

**直接関係なし。** Day は日付コンテナ。Participant は Trip 全体に同行する人として登録し、特定 Day に限定しない（v2）。

### Itinerary

**直接関係なし（v2）。** 行動は Itinerary が正本。Participant は「その Trip に誰がいるか」の集合。

将来、Itinerary ごとに参加者サブセットを持つ必要が出た場合は **v2 スコープ外** として別検討する。

### Expense

**v2 では構造的に紐づけない。** 関係は **v3 Shared Expense** で初めて確立する。

| 観点 | v2 | v3（予定） |
|---|---|---|
| `paid_by_name` | **維持** — 文字列記録のまま | Participant 名と **手動整合** または backfill |
| `paid_by_participant_id` | **列なし / 未使用** | Expense に FK 追加 |
| beneficiary / 按分 | **なし** | `expense_beneficiaries` 等 |
| 金額正本 | **Expense のみ**（v1.22 維持） | 変更なし |

v2 で Participant を導入しても、**既存 Expense 行の意味は変わらない**。`paid_by_name` は引き続き有効な interim 表現である。

運用上、ユーザーが `paid_by_name` に Participant と同じ表示名を書くことは **許容** するが、**システムは v2 では自動解決しない**。

### Reservation

[reservation-responsibilities-review.md](reservation-responsibilities-review.md) の方針を **維持**:

```text
Reservation と Participant は v2 では直接リンクしない。
```

| 観点 | 方針 |
|---|---|
| 予約名義 | Reservation の `provider_name` / 確認情報が正本 |
| 「誰の部屋か」 | v2 では Note または Reservation remark で足りる |
| 将来 | ホテル予約と Participant の紐づけは **optional 拡張**（Open Questions） |

### Note

| 関係 | 方針 |
|---|---|
| **エンティティ** | 別物 — Note は Annotation、Participant は **Trip 内の参加関係** の正本 |
| **本文での言及** | 「長男向けの注意」等は Note に書いてよい |
| **Participant 専用メモ** | v2 では Participant 行に optional 短い note 列を持つかは #8 で決定 |

### Summary

| 関係 | 方針 |
|---|---|
| **責務** | Trip / Day の Abstract — 参加者リストの正本ではない |
| **生成入力** | 将来 Summary Generator が Participant 数・構成を **参照しうる** |
| **混同禁止** | Summary 本文に同行者一覧を **正本として** 書かない |

### Checklist

| 関係 | 方針 |
|---|---|
| **スコープ** | ともに Trip 配下だが **別エンティティ** |
| **担当者割当** | v2 では **なし** — 「誰のパスポート」等は Checklist テキストまたは Note |
| **将来** | Checklist item に `assigned_participant_id` は v3 以降の検討 |

---

## v2.0.0 Scope

### 実施する（設計系列を通じた到達像）

| # | 内容 | フェーズ |
|---|---|---|
| 1 | 本 Responsibilities Review | **#7（本書）** |
| 2 | フィールド・DB・export 詳細 | #8 Entity Design |
| 3 | 実装計画・テスト方針 | #9 Implementation Plan |
| 4 | `participants` テーブル + CLI CRUD | #10 Implementation |
| 5 | export **schema v4**（`participants[]`） | #10 Implementation |
| 6 | import / validate-export / duplicate 対応 | #10 Implementation |
| 7 | Post-Implementation Review | #11 |
| 8 | Release v2.0.0 | #12 |

### v2.0.0 の機能スコープ（責務上の約束）

```text
✓ Trip に参加者（自分を含む）を登録・一覧・更新・削除できる
✓ `is_self` でこの Trip における自分をマークできる（最大 1 件）
✓ Participant は Trip スコープの **参加関係** の正本データである（人そのものの正本ではない）
✓ statistics では participant_count と companion_count を混同しない（§Participant count semantics）
✓ export v4 で Participant をバックアップ・移行できる
✓ v3 で Expense 紐づけに使える安定 ID を持つ

✗ 精算・割り勘・beneficiary
✗ Expense への FK / 自動名前解決
✗ Reservation への Participant リンク
✗ User アカウント・権限
```

### 想定のユーザー価値

```text
旅行計画時に「誰が行くか」（自分を含む）を CLI / 将来 GUI で明示できる。
グループ旅行の export に参加者リストが含まれる。
v3 以前から paid_by_name と表示名を揃える運用が可能（任意）。
```

---

## Deferred Scope

### v3 Shared Expense

| 項目 | 内容 |
|---|---|
| **テーマ** | 誰が払ったか / 誰の費用か |
| **Expense 拡張** | `paid_by_participant_id`、beneficiary、Settlement |
| **v2 との関係** | v2 Participant ID が **参照先の正本** |
| **export** | v4 上に optional フィールド追加、または v5 — #9 で決定 |

[long-term-version-strategy.md](../long-term-version-strategy.md) §v3:

```text
ここで初めて Expense が Participant と結び付く。
```

### v5 Travel Book

しおりに同行者セクションを載せるのは **表示・生成レイヤー** の責務。v2 は **データ正本** のみ。export-md / Travel Book Generator が v2 Participant を入力に読むのは自然。

### v6 Travel Journal

旅行記・Photo は Participant と **関連しうる** が、Journal 実装は v6。v2 Participant は **誰がいたか** の事実のみ。

### その他の意図的 defer

```text
Budget / Estimate
Venue 正本
ISO 4217 厳格化 + 換算
Participant ごとの権限
クラウド同期
```

---

## Export / Import Considerations

**本フェーズ（#7）では export schema を変更しない。** 将来像のみ記録する。

### Export schema v4 候補

| 論点 | 推奨方針 |
|---|---|
| **バージョン** | `schema_version: 4` |
| **配置** | **top-level `participants[]`**（Trip 直下の兄弟） |
| **理由** | Participant は Trip スコープ。Day / Itinerary ネストより top-level が自然 |
| **内部 ID** | export しない（Expense / Reservation と同型） |
| **安定参照** | `display_name` + `sort_order`、または export 専用 `participant_ref` — #8 で確定 |
| **v3 互換** | v3 import **継続**。v4 export に `participants[]` 追加 |
| **Expense** | v4 では `paid_by_name` **維持**。`paid_by_participant_ref` は **v3 まで保留** |

### 構造イメージ（案）

```json
{
  "schema_version": 4,
  "trip": { "name": "沖縄旅行", "start_date": "...", "end_date": "..." },
  "participants": [
    { "name": "Alex", "sort_order": 0, "is_self": true },
    { "name": "Jordan", "sort_order": 1, "is_self": false }
  ],
  "days": [ ],
  "checklist_items": [ ],
  "notes": [ ]
}
```

### import 順序（案）

```text
Trip → Participants → Days / Itineraries → … → Expenses
```

Participant を Expense より **先に** 作成し、v3 以降の ref 解決に備える。

### duplicate / roundtrip

`trip duplicate` で Participant も複製するのが期待動作。#10 でテスト化。

---

## CLI Considerations

**本フェーズでは CLI を実装しない。** 将来コマンド体系のたたき台:

```bash
participant add    --trip <trip_id> --name "Alex"
participant list   --trip <trip_id>
participant show   <participant_id>
participant update <participant_id> --name "..."
participant delete <participant_id>
```

| 論点 | 方針 |
|---|---|
| **owner** | `add` / `list` は `--trip` 必須（Note の Trip パターンに近い） |
| **ID** | `show` / `update` / `delete` は **Participant 行 ID**（DB 全体で一意。Person マスター ID ではない） |
| **`--json`** | `list` / `show` で対応（既存エンティティと同型） |
| **Expense 連携** | v2 CLI に `expense --paid-by-participant` は **入れない**（v3） |
| **名前の一意性** | 同一 Trip 内で `display_name` 重複を許すか — #8 Open Question |

コマンドのオプション詳細・エラーメッセージ・cascade は **Entity Design / Implementation Plan** へ送る。

---

## Compatibility Considerations

### 既存 DB

| 項目 | v2 導入時 |
|---|---|
| Migration | **新規 `participants` テーブル追加**（#10） |
| 既存 Trip | Participant 0 件のまま — backfill 不要 |
| Expense / Reservation | **列変更なし**（v2） |

### 既存 export

| From | To | 互換 |
|---|---|---|
| v3 export | v4 import | `participants` 省略 = 空配列 |
| v4 export | v3 import | **不可**（v3 は participants 未知）— 想定どおり |
| v3 export | v3 import | **継続** |

### canonical sample

`okinawa_sesoko_2026` への Participant 投入は **#10 以降の任意タスク**。Responsibilities Review では必須としない。

### `paid_by_name` との共存

v2 リリース後も:

```text
Expense.paid_by_name は引き続き有効。
Participant との自動リンクは v3 まで行わない。
```

---

## Open Questions

Entity Design（#8）で解決する項目:

| # | 質問 |
|---|---|
| 1 | 同一 Trip 内で `display_name` の **一意性** を要求するか |
| 2 | Participant に **optional `note`** 列を持つか（Note entity との境界） |
| 3 | export での安定キー — `display_name` のみか、`participant_ref` UUID 風文字列か |
| 4 | `trip duplicate` での Participant ID 再採番と Expense `paid_by_name` の関係（手動運用で足りるか） |
| 5 | 将来 Reservation ↔ Participant の **optional** リンク要否 |
| 6 | Checklist 担当者割当の時期（v3 以降でよいか） |

---

## Completion Criteria

本 Responsibilities Review（Issue #7）の完了条件:

| # | 条件 | 状態 |
|---|---|---|
| 1 | `participant-model.md` が存在する | 本書 |
| 2 | Responsibilities / Non-responsibilities が明確 | §Responsibilities, §Non-responsibilities |
| 3 | v2.0.0 scope と deferred scope が明確 | §v2.0.0 Scope, §Deferred Scope |
| 4 | v3 Shared Expense との境界が明確 | §Expense, §Deferred v3 |
| 5 | Export / CLI の将来影響を整理 | §Export, §CLI |
| 6 | Rust / DB / export 実装なし | 本フェーズ対象外 |
| 7 | 次フェーズは Entity Design（#8） | 上記 Open Questions を引き継ぎ |

---

## Next phase notes（Implementation #10）

[participant-implementation-plan.md](participant-implementation-plan.md)（#9）で実装手順を確定済み。#10 でコード変更。

実装・テスト・Release は #10–#12 に従う。
