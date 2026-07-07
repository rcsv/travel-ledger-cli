# Venue Model — Introduction Policy

Travel Ledger に **Venue（地理的な場所）** を導入する前の、責務・スコープ・正規化方針を定める仕様書です。

| Item | Status |
|---|---|
| Phase | **Planning — 実装前** |
| Version track | **v4.7.x リリース対象外**（Proposal / Fragment apply 系とは別軸） |
| Implementation | **未着手**（DB / CLI / export schema 変更なし） |
| Related | [itinerary-model.md](itinerary-model.md) · [reservation-model.md](reservation-model.md) · [reservation-entity-design.md](reservation-entity-design.md) |

---

## 1. 背景と維持する原則

Caglla / Travel Ledger では v1.8.0 以降、**Itinerary is not a venue** を設計原則としてきました。

| 概念 | 意味 |
|---|---|
| **Itinerary** | 旅行中の **行動単位**（何をするか・いつ行うか） |
| **Venue** | **地理的な場所**（施設・POI・住所可能な地点） |

本書はこの責務分離を **維持したまま**、Itinerary が Venue を **任意参照** できる初期方針を定めます。

```text
Itinerary represents a travel activity.
Venue represents a geographic place.
An itinerary item may optionally reference a Venue.
```

---

## 2. 初期 Venue model のスコープ

### 2.1 In scope（初期方針）

```text
Initial Venue model scope:

- Venue represents a geographic place.
- Itinerary remains a travel activity, not a venue.
- An itinerary item may reference at most one primary Venue.
- The primary Venue represents the main place associated with the activity.
- origin / destination / waypoint / pickup / return roles are intentionally out of scope.
- Route-like information should remain as itinerary text, note, or location_text.
- If a stop itself is a meaningful travel activity, it should be represented as a separate Itinerary.
```

日本語要約:

- Itinerary は旅行中の行動単位であり、Venue は地理的な場所を表す。
- 初期 Venue モデルでは、Itinerary は **最大 1 つの primary Venue** のみを参照できる。
- primary Venue は、その行動における **主な場所** を表す。
- `origin` / `destination` / `waypoint` / `pickup` / `return` などの **複数 Venue role** は初期スコープ外とする。
- 移動経路や経由地は、当面 Itinerary の `title` / `note` / `location`（location_text）に保持する。
- 経由地での滞在や行動が意味を持つ場合は、**別 Itinerary** として表現する。

### 2.2 体験・スキーマ・将来の整理

| 層 | 初期方針 |
|---|---|
| **Experience** | 場所を **1 つ選ぶだけ**（primary venue） |
| **Schema** | **primary venue ref only**（単一参照） |
| **Text** | 経路・補足は **自然文**（title / note / location）で保持 |
| **Future** | 複数地点の移動構造は **Route / Segment / Transport Leg** として **別途** 検討 |

Venue model を早期に肥大化させず、移動の構造化が必要になったときは **別エンティティ族** で扱う。

### 2.3 Out of scope（初期）

| 項目 | 理由 |
|---|---|
| 1 Itinerary あたり複数 Venue ref | Fragment / import の正規化が不安定になる |
| origin / destination / waypoint / pickup / return role | 同一内容の複数正規表現を許す |
| Venue 内の Routing / 経路計算 | [reservation-model.md §8](reservation-model.md#8-routing-は対象外) と同型で対象外 |
| Reservation への Venue 複製 | [reservation-entity-design.md §10](reservation-entity-design.md#10-venue-参照方針) を維持 |
| Maps provider / POI 自動解決の必須化 | 任意メタデータ・将来拡張 |
| GUI / Travel Book 専用 Venue 章 | UI 要件確定まで Defer（v4.4.x 結論と整合） |

---

## 3. なぜ primary Venue のみか

### 3.1 正規化のぶれ

同じ旅行内容を、複数の粒度で表現できてしまうと、**Fragment Apply** や **AI-driven proposal / import** の正規化が不安定になります。

例: 「那覇空港から万座毛を経由してヒルトン瀬底へ行く」

| 表現 | 内容 | 初期方針での扱い |
|---|---|---|
| **1. 複数 Itinerary** | 出発 / 万座毛立ち寄り / ヒルトン瀬底へ移動 | **推奨** — 行動単位が明確なら分割 |
| **2. 1 Itinerary + 複数 Venue role** | origin / waypoint / destination | **初期スコープ外** |
| **3. 1 Itinerary + primary Venue のみ** | title + primary: ヒルトン瀬底、note: 経路テキスト | **許容** — 主な場所は 1 つ、経路はテキスト |

初期方針は **2 を禁止** し、**1 と 3 の使い分け** を仕様で誘導します。

### 3.2 判断ルール（初期）

```text
主な滞在・訪問先が 1 つに定まる
  → 1 Itinerary + primary Venue（任意）+ note/location に経路補足

経由地での滞在・観光・食事などが独立した行動
  → 経由地ごとに別 Itinerary（それぞれ primary Venue 可）

移動そのものが主目的で経路だけが重要
  → 1 Itinerary（category: transport 等）+ title/note/location に経路テキスト
  → primary Venue は任意（到着地のみ付けるか、付けないかは人間判断）
```

---

## 4. 現行フィールドとの関係

現行 CLI / export schema v8 では、Itinerary は **Venue entity を持たず**、任意の `location` 文字列のみです（[itinerary-model.md §1](itinerary-model.md#venue--place-is-optional-metadata)）。

| 現行 | 将来（初期 Venue 導入後の想定） |
|---|---|
| `location`（自由文字列、任意） | **移行期の表示用テキスト**として維持可能 |
| （Venue ref なし） | **optional `primary_venue_ref`**（名前・形は実装フェーズで確定） |
| `title` / `note` | 経路・補足の **正本は引き続きテキスト可** |

Venue 導入時も **Itinerary is not a venue** は変えません。Venue は Itinerary に **任意で紐づく補助 entity** です。

Reservation は引き続き施設正本を複製せず、Itinerary（および将来の Venue）から辿る — [reservation-entity-design.md §10](reservation-entity-design.md#10-venue-参照方針)。

---

## 5. Proposal / Fragment への含意

v4.7.x の Proposal Fragment apply（`add` / `update_itinerary` 等）では、Itinerary の in-place 属性（`title` / `note` / `location` 等）のみを扱っています。

Venue 導入 **後** の Fragment 拡張を検討する場合も、初期方針は同型とします。

| 観点 | 初期方針 |
|---|---|
| Fragment candidate | **primary venue ref は 0 または 1** |
| 複数 venue role | **reject / normalize しない**（そもそもスキーマに入れない） |
| 経路・経由 | `note` / `location` / `title` に残す |
| 経由地の独立行動 | **別 Fragment（add itinerary）** として分割を推奨 |
| `update_itinerary` | primary venue の付け替えは **将来フェーズ**（v4.7.28 時点では未実装） |
| `delete_itinerary` | Venue link / snapshot / provider cache は **delete blocker にしない** — [v4.7.30 P-6j policy §2.2.1](v4.7.30-p6j-destructive-structural-apply-policy.md#221-venue--place--not-a-delete-blocker) |

**正規化の安定性** を優先し、AI が「1 行動 = 1 primary place」に収束しやすい契約を先に固定します。

---

## 6. 将来: Route / Segment / Transport Leg

移動の **構造化**（出発地・到着地・経由・交通手段・時刻幅）が必要になった場合:

```text
Venue model を肥大化させない
  → Route / Segment / Transport Leg として別モデルで検討
```

| モデル（案） | 役割 |
|---|---|
| **Venue** | 地点そのもの（変更なし） |
| **Route / Segment / Transport Leg** | 地点間の移動・区間・交通（将来） |
| **Itinerary** | 旅行者の行動単位（変更なし） |

Routing は [reservation-model.md](reservation-model.md) でも Reservation 対象外とされており、本書の将来方針と整合します。

---

## 7. バージョン配置の判断

| 判断 | 内容 |
|---|---|
| **v4.7.28 / v4.7.29 とは別** | 現行マイルストーンは P-6i `update_itinerary`（Fragment apply）。Venue は **データモデル横断** の別トラック |
| **本書の位置づけ** | **実装前の設計正本** — tag / Cargo version 変更なし |
| **実装着手の目安** | export schema / DB migration / CLI の **専用計画書** を別途作成後（v4.8+ 候補） |
| **Travel Book / public schema** | schema v8 変更前に [export-schema.md](export-schema.md) と public docs を同期 |

---

## 8. Non-goals（本書）

```text
DB migration
venues テーブル / CLI 追加
export schema v8 変更
Fragment intent 拡張
Maps / POI API 連携
Travel Book Venue 章
Route / Segment 実装
```

---

## 9. 次の文書候補（実装フェーズ — 未作成）

| 順 | 候補 | 内容 |
|---|---|---|
| 1 | **Venue Entity Design** | フィールド、ID、POI メタ、Trip スコープ vs グローバル |
| 2 | **Venue Implementation Plan** | DB / CLI / export / import / golden |
| 3 | **Venue × Fragment** | `add` / `update_itinerary` での primary venue ref |
| 4 | **Route / Segment concept** | Venue とは別軸の移動構造化（必要時のみ） |

---

## 10. 用語

| 用語 | 意味 |
|---|---|
| **Venue** | 地理的な場所・施設・POI |
| **primary Venue** | 1 行動に紐づく **主な** 場所（初期はこれのみ） |
| **location / location_text** | Itinerary 上の自由文字列（移行期・経路補足含む） |
| **Route / Segment / Transport Leg** | 将来検討の移動構造（本書では未定义） |

---

## 11. 参照

| ドキュメント | 関係 |
|---|---|
| [itinerary-model.md](itinerary-model.md) | Itinerary is not a venue · location 任意 |
| [reservation-model.md](reservation-model.md) | Venue / Routing の責務分離 |
| [reservation-entity-design.md](reservation-entity-design.md) | Reservation は Venue を複製しない |
| [planning-design-principles.md](planning-design-principles.md) | 横断設計原則 |
| [v4.7.3 Proposal Fragment concept spec](v4.7.3-proposal-fragment-concept-spec.md) | Fragment 正規化の文脈 |
| [long-term-version-strategy.md](../long-term-version-strategy.md) | Venue / Maps defer 履歴 |
