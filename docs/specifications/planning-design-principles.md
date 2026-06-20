# Planning Design Principles

Caglla.Travel CLI における **旅行計画（Planning Foundation）** の設計思想 — エンティティの役割分担と、あえて持たせないフィールドの理由。

**位置づけ:**

```text
仕様変更ではない — 現行実装（export schema v4 含む）が意図どおりであることの明文化
v3 Shared Expense 等、将来の判断軸として読み返すための短文書
```

| 関連 | 参照 |
|---|---|
| Itinerary | [itinerary-model.md](itinerary-model.md) |
| Checklist | [checklist-design-memo.md](checklist-design-memo.md) |
| Note | [note-model.md](note-model.md) |
| Reservation / Expense | [reservation-model.md](reservation-model.md) · [expense-model.md](expense-model.md) |
| Travel Ledger 横断 | [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) |

---

## 1. Caglla が想定する旅行

Caglla は **個人・家族・友人との旅行計画** を支援する。旅行を **業務オペレーション** や **タスク消化** の対象にはしない。

```text
旅行計画を支援する          ← Caglla の主眼
旅行をタスク管理する        ← 主眼ではない
```

社内イベントで関係会社ブースを 5 分刻みで回るような用途なら、Itinerary をタスクリスト化する設計もあり得る。しかし Caglla の主対象ではないため、**Itinerary に完了チェック（`is_done`）を載せない** など、計画体験をシンプルに保つ選択をしている。

---

## 2. Itinerary is not a task row

**Itinerary は旅行行動の単位であり、タスク管理の行ではない。**

| 観点 | 方針 |
|---|---|
| 表すもの | 旅行中の **行動の流れ** — 予定・移動・立ち寄り |
| 中心体験 | 時系列で「何をするか」を並べる。各項目を **完了／未完了とチェックしていく** 体験は中心にしない |
| `is_done` | **持たない** — 意図的な設計（抜けではない） |
| 関連原則 | [Itinerary is not a venue](itinerary-model.md#itinerary-is-not-a-venue) · [Itinerary is a unit of travel activity](itinerary-model.md#itinerary-is-a-unit-of-travel-activity) |

export schema v4 の JSON でも、Itinerary 配下に `is_done` はなく、Checklist の `checklist_items[]` だけが `is_done` を持つ — この非対称は上記思想の反映である。

---

## 3. Checklist is the place for confirmation tasks

**確認したいことは Checklist に置く。** ここが `is_done` の自然な置き場所。

| 例 | Checklist でよい理由 |
|---|---|
| 持ち物 | 出発前に **チェックしたい** |
| 予約確認 | 忘れ防止の **確認タスク** |
| 書類・準備 | 旅行前後の **To-do 的確認** |
| 事前確認 | 明示的に ✓ を付けたい項目 |

Checklist は Trip 配下（Preparation Layer — [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md)）。Itinerary 行を増やして「パスポート確認」を埋め込むのではなく、**確認事項は Checklist、行動は Itinerary** と分ける。

将来、担当者割当（`assigned_participant_id` 等）は v3 以降の検討事項。現行 v2 でも **Checklist = 確認タスク、`is_done` あり** という分離は維持する。

---

## 4. Notes remain flexible

**Note は自由記述の逃げ場として残す。**

| 方針 | 理由 |
|---|---|
| すべてを構造化しない | 入力項目が増えると、普通の旅行計画では使いにくくなる |
| 構造化は価値がある場合だけ | Reservation / Expense / Checklist など、**明確に役割がある** ときだけ型を増やす |
| Long-form テキスト | Trip / Day / Itinerary に紐づく **Annotation** — [note-model.md](note-model.md) |

「この情報、どのフィールドに入れる？」と迷ったとき、無理に Itinerary や Checklist に押し込まず **Note で足りる** 設計を許容する。v3 Shared Expense でも、精算に必要な構造だけを増やし、雑多なメモまでフィールド化しない — 詳細は [shared-expense-model.md](shared-expense-model.md) §Note vs structured fields。

---

## 5. Reservation multiplicity — allowed, granularity hint

**1 Itinerary に複数 Reservation を紐づけることは現行仕様で許容している。** 制約ではない。

| 観点 | 方針 |
|---|---|
| 自然な形 | 通常は **0 or 1** Reservation / Itinerary |
| 複数が付くとき | それぞれが **独立した旅行行動** なら、Itinerary を **分割した方が自然** な可能性 |
| 現行 CLI | 複数を **拒否しない** — データモデル・import / export は配列で受け付ける |
| 将来 | **doctor / advisor の観点候補**（warning レベル）。自動分割や cardinality 制約は **今回導入しない** |

例: 「午前の美術館」と「午後のレストラン予約」を 1 行の Itinerary に両方載せるより、行動単位で 2 行に分ける方が計画として読みやすい — そういう **設計上のヒント** として扱う。

---

## 6. Multiple expenses under one itinerary — natural

**Expense は Reservation と異なり、1 Itinerary に複数紐づくことが自然にあり得る。** Itinerary 粒度が粗い **サインとは限らない**。

| 例 | 説明 |
|---|---|
| フードコート | 各自が別店舗で購入 → 複数 Expense |
| レストラン | 食事代と追加ドリンクが別会計 |
| 買い物立ち寄り | 複数店舗で支払い |
| レンタカー行動 | 高速代・駐車場・給油が同一「移動」行内で発生 |

Expense は **Transaction Record Layer**（[expense-model.md](expense-model.md)）。同一行動の中で複数の支払いが起きるのは日常的である。v3 Shared Expense（`paid_by_participant_id` 等）でも、**1 Itinerary : N Expense** の関係は維持される想定。

Reservation の「複数は粒度ヒント」と混同しない:

```text
Reservation 複数 → 行動分割を検討するヒントになりやすい
Expense 複数     → 同一行動内の複数取引として自然
```

---

## 7. 判断に迷ったとき

| 質問 | 置き場所の目安 |
|---|---|
| 旅行中に **いつ・何をするか**（行動の流れ） | **Itinerary**（`is_done` なし） |
| 出発前に **✓ したい確認** | **Checklist**（`is_done` あり） |
| 自由メモ・長文・迷った情報 | **Note** |
| 予約・チケットの **Booking Record** | **Reservation**（Itinerary に 0–1 が自然、複数は許容） |
| 実際に払った **金額** | **Expense**（Itinerary に複数可） |

---

## 8. 明示的に今回やらないこと

本書は **documentation-only**。以下は変更しない:

- Itinerary に `is_done` を追加しない
- Reservation の cardinality 制約を追加しない
- Note / Checklist の入力制約を増やさない
- export schema version bump
- v3 Shared Expense の設計・実装

---

## 参照

| 領域 | ドキュメント |
|---|---|
| v1 総括 | [planning-foundation-completion-review.md](planning-foundation-completion-review.md) |
| v3 前点検 | [foundation-hardening-review.md](foundation-hardening-review.md) |
| ロードマップ | [long-term-version-strategy.md](../long-term-version-strategy.md) §v3 Shared Expense |
