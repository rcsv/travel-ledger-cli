# Reservation Responsibilities Review（責務整理 — 実装後レビュー）

Caglla.Travel CLI の **Travel Ledger Model** における **Reservation** の責務を、**v1.18.0 実装後** に整理・検証するレビューです。

**v1.19.0 時点: 仕様整理のみ（v1 Hardening 第一弾）。** 本書は実装変更を伴わない。改善候補は §12 に記録する。

| ドキュメント | 役割 |
|---|---|
| [travel-ledger-responsibilities.md](travel-ledger-responsibilities.md) (v1.10.0) | Summary / Remark / Note / Reservation の横断比較 |
| [reservation-model.md](reservation-model.md) (v1.11.0) | 責務・境界（What it is / is not） |
| [reservation-entity-design.md](reservation-entity-design.md) (v1.12.0) | フィールド・種別・拡張戦略 |
| [reservation-implementation-plan.md](reservation-implementation-plan.md) (v1.13.0) | 実装計画 |
| **本書** (v1.19.0) | **実装後**の責務検証 — 保存・表示・連携・将来関係 |

関連: [summary-responsibilities-review.md](summary-responsibilities-review.md) / [note-model.md](note-model.md) / [expense-model.md](expense-model.md) / [export-schema.md](export-schema.md)

設計系列:

```text
v1.11.0  Responsibilities (reservation-model)
v1.12.0  Entity Design
v1.13.0  Implementation Plan
v1.18.0  Implementation
v1.19.0  Responsibilities Review  ← this document (post-implementation)
```

---

## 1. Goals（実装が満たすべき責務）

| 課題 | v1.18.0 の解決 |
|---|---|
| **予約番号を構造化して保持** | `confirmation_code` — Remark への埋め込みに依存しない |
| **予約サイト・確認 URL を保持** | `reservation_site_url` |
| **契約上の期間を記録** | `start_at` / `end_at`（任意）— Itinerary 時刻とは別軸 |
| **行動と予約情報を分離** | Itinerary = 何をするか、Reservation = 実行に必要な予約・確認 |
| **しおりで予約一覧** | `reservation list --trip`、`trip export-md` の `## Reservations` |
| **バックアップ・移行** | export v3 `days[].itineraries[].reservations[]`、import roundtrip |
| **変更の検出** | `trip diff` — added / removed / modified |

Reservation は **読む人が旅行前に手続き情報を探す** ための概念。旅程の時系列正本（Itinerary）でも、費用正本（Expense）でもない。

---

## 2. Non-goals（引き続き扱わない）

| 概念 | 理由 | 正しい置き場 |
|---|---|---|
| **費用・領収書** | 予約の有無と独立 | **Expense** |
| **長文の背景・経緯** | 自由記述 | **Note entity** |
| **行内の短文補足** | 旅程表の備考 | **Remark**（`itinerary_items.note`） |
| **施設・POI 正本** | 住所・電話の正本は Venue 側 | Itinerary `location` / 将来 **Venue** |
| **移動経路** | 空間的幅 | 将来 **Routing**（Itinerary 行で表現） |
| **旅行全体の要約** | 読者向け概要 | **Trip / Day Summary** |
| **同行者・精算** | 誰が支払ったか | 将来 **Participant** |
| **写真・添付** | メディア | 将来 **Photo / Attachment** |

---

## 3. 保存責務と表示責務（二層構えの検証）

設計時の方針:

```text
保存:  Itinerary 配下（reservations.itinerary_id）
表示:  Trip 単位で集約（一覧・しおり）
```

### v1.18.0 実装との整合

| 層 | 実装 | 判定 |
|---|---|---|
| **保存** | `reservations.itinerary_id NOT NULL`。`trip_id` は持たない | **設計どおり** |
| **辿り方** | `reservation → itinerary_items → days.trip_id` | **設計どおり** |
| **集約表示** | `reservation list --trip`、`trip export-md` | **設計どおり** |
| **局所表示** | `reservation list --itinerary`、`itinerary show` インライン | **設計どおり** |

Trip 直下の `reservations[]`（トップレベル export）は **採用していない**。これは Entity Design の Itinerary ネスト方針と一致し、**正しい判断** とする。

### 表示モデル vs 保存モデル

```text
Trip 単位の予約一覧 = 表示モデル・集約ビュー（正本ではない）
正本 = Itinerary 配下の Reservation 行
```

GUI / Web 版でも、利用者が「Trip の契約台帳」を直接編集する UI ではなく、**Itinerary 文脈から予約を追加し、Trip では読む** 形を維持する。

---

## 4. Reservation と他概念の境界

### 4.1 Reservation と Remark

| | **Reservation** | **Remark** |
|---|---|---|
| **責務** | 予約・確認の構造化正本 | 行動の短文補足 |
| **例** | Confirmation ABC123、予約 URL | 要: ETCカード、チェックアウトリミット 10:00 |
| **件数** | 0..N per Itinerary | 0..1 per Itinerary |
| **しおり** | Trip 集約セクション | Itinerary 行内 |

**移行期:** okinawa canonical sample では予約番号が Remark に残っている。**許容** するが、新規データは Reservation を正とする。

### 4.2 Reservation と Note

| | **Reservation** | **Note** |
|---|---|---|
| **責務** | 予約番号・手続きの正本 | 背景・経緯・振り返り |
| **構造** | 型付きフィールド | `title` + `body` 自由記述 |
| **検索** | type / provider で一覧可能 | 全文検索向き |

同一 Itinerary に両方存在しうる。予約サイトでのやり取りのログは **Note**、確認番号は **Reservation**。

### 4.3 Reservation と Expense

予約と費用は **独立**。ホテル宿泊で Reservation（予約番号）と Expense（宿泊費）が共存する典型例。

### 4.4 Reservation と Summary

| | **Summary** | **Reservation** |
|---|---|---|
| **読者** | 旅行の意図・その日のテーマ | 手続き・確認情報 |
| **しおり** | Trip/Day 冒頭 | `## Reservations` 集約セクション |
| **混在禁止** | Summary に予約番号を書かない | Reservation に旅行全体の要約を書かない |

### 4.5 Itinerary 時刻との関係

| 情報 | 正本 |
|---|---|
| 旅程上の「何時ごろ」 | Itinerary `start_time` + Sequence |
| 契約・予約上の期間 | Reservation `start_at` / `end_at`（任意） |

**二重管理の懸念:** ホテル check-in 時刻が Itinerary と Reservation の両方に載りうる。  
**方針:** 旅程表は Itinerary、予約確認・契約区間は Reservation。**完全一致を強制しない**（v1.18.0 MVR の妥当な割り切り）。

---

## 5. Export / Import（schema v3）

### 実装

```json
{
  "days": [
    {
      "itineraries": [
        {
          "reservations": [
            {
              "reservation_type": "hotel",
              "provider_name": "Hilton Okinawa Sesoko Resort",
              "confirmation_code": "ABC123"
            }
          ]
        }
      ]
    }
  ]
}
```

| 論点 | v1.18.0 実装 | レビュー |
|---|---|---|
| ネスト位置 | `itineraries[].reservations[]` | Implementation Plan 案どおり |
| 空配列 | export 時省略 | okinawa golden 互換 — **適切** |
| 後方互換 | キーなし → 空 | **適切** |
| 内部 ID | export しない（親子構造で関連） | Expense と同型 — **適切** |
| `trip duplicate` | roundtrip で複製 | **適切** |
| `validate-export` | type / provider 検証 | **適切** |

### 未実装フィールド（export に含まれない）

Entity Design の MVR 超過分は **意図的に省略**（v1.18.0）:

```text
reservation_site_name
contact_name / contact_phone / contact_email
website
flight / rental_car 固有 details
```

schema v4 が必要になる条件は [reservation-implementation-plan.md §7](reservation-implementation-plan.md#7-export--import) と同様 — **現時点では v3 拡張で十分**。

---

## 6. Markdown export（Travel Book との関係）

### 実装（`trip export-md`）

- Trip 全体の **`## Reservations`** セクション（0 件なら省略）
- 種別見出し（`### Hotel` 等）
- **Day / Itinerary 文脈** を各行に表示
- Itinerary 節内へのインライン Reservation は **未実装**（集約セクションのみ）

### Travel Book（しおり）としての評価

| 観点 | 状態 |
|---|---|
| 旅行前の予約一覧 | **満たす** — Trip 集約セクション |
| どの行動の予約か | **満たす** — Day / Itinerary ラベル付き |
| Day ごとの旅程表内表示 | **未実装** — 将来候補（§12） |
| 印刷向けテーブル形式 | **未実装** — 現行は見出し + 箇条書き |

**Travel Book** を製品メジャーで語る場合、v1.18.0 の Markdown は **第一弾の集約セクション** であり、Day/Itinerary インライン統合は Hardening 以降の改善候補とする。

---

## 7. Validation

### 実装済み

| ルール | 実装 |
|---|---|
| `itinerary_id` 存在 | add / import 時に検証 |
| `reservation_type` 必須・既知種別 | CLI + validate-export |
| `provider_name` 必須 | CLI + validate-export |
| 任意テキスト trim / 空 → NULL | アプリ層 |

### 未実装（改善候補 — §12）

| ルール | 深刻度 |
|---|---|
| `start_at` > `end_at` | Warning（doctor 候補） |
| `confirmation_code` 欠落 | Info（doctor 候補） |
| 未知 type が DB に存在 | Warning（import 破損検出） |
| Remark に予約番号らしき文字列 + Reservation なし | Suggestion（移行促進） |

---

## 8. Diff

### 実装

```text
+ Reservation added
- Reservation removed
~ Reservation modified
```

**比較キー:** Itinerary コンテキスト（day_number, sort_order, start_time, title）+ `reservation_type` + `provider_name` + `confirmation_code`。

### レビュー

| 観点 | 判定 |
|---|---|
| 同一 Itinerary に同 type/provider で confirmation のみ異なる2件 | 別キーとして検出 — **妥当** |
| confirmation 変更 | remove + add ではなく **modified** — **妥当** |
| field 変更 | remark / url / start_at / end_at を検出 — **妥当** |
| Expense diff | v1.18.0 では **未実装**（Reservation と同型の将来候補） |

---

## 9. CLI 責務（v1 Hardening 観点）

| コマンド | 責務分類 | 備考 |
|---|---|---|
| `reservation list --trip` | **確認・連携** | しおり向け集約 — 優先度高 |
| `reservation show` / `--json` | **確認・連携** | ツール連携 |
| `reservation add/update/delete` | **手入力編集** | 現行サポート。将来 GUI 成熟後に deprecated 候補になり得る（[long-term-version-strategy.md](../long-term-version-strategy.md)） |
| `trip export` / `import` / `diff` / `validate-export` | **確認・連携** | v1 基盤の中核 |

v1.18.0 の CLI スコープは **Implementation Plan の MVR と一致**。過剰な種別固有フラグは **意図的に省略** してよい。

---

## 10. Canonical Sample

| 項目 | 方針（継続） |
|---|---|
| `okinawa_sesoko_2026` | **Reservation 投入なし** — 清算・export golden の主目的を維持 |
| `samples/reservation_sample_commands.sh` | 独立デモ — **適切** |
| Remark 上の予約情報 | 移行期の現実的データとして **残す** |

canonical への段階投入は **Hardening 後の別判断**（しおり検証 Trip の新設を推奨）。

---

## 11. Future Relationship

### 概念マップ（v1.18.0 到達点）

```text
Trip
 ├─ summary              ← v1.17.0
 ├─ Note[]
 ├─ Checklist
 └─ Day
      ├─ summary         ← v1.17.0
      ├─ Note[]
      └─ Itinerary
           ├─ title / remark
           ├─ Expense
           ├─ Reservation ← v1.18.0（Itinerary 配下保存）
           └─ Note[]
```

### Participant（将来 v2）

| 関係 | 方針 |
|---|---|
| Reservation と Participant | **直接リンクなし**（v1.18.0） |
| `paid_by_name`（Expense） | Participant 導入後に精算へ移行候補 |
| 予約名義・ゲスト名 | 現行は `remark` または Note。将来 `guest_names` 等は Entity Design 拡張で検討 |

Reservation は **誰の予約か** より **何の予約か・確認番号は何か** に集中。Participant は精算・同行者文脈。

### Travel Book（しおり / 計画共有）

| 情報 | v1.18.0 のソース |
|---|---|
| 旅行の狙い | Trip **Summary** |
| 主な行先 | Day **Summary** |
| 予約番号・手続き | **Reservation** 集約セクション |
| 旅程 | Itinerary 列 + Remark |
| 費用 | Expense（Itinerary 内 / 集計は stats） |
| 忘れ物 | Checklist |

**Travel Book 完成度:** Summary + Reservation + Itinerary + Checklist で **旅行前共有の骨格は成立**。Photo / Participant / Venue は将来レイヤ。

### Photo / Attachment（将来）

予約確認メール PDF、航空券画像は **Attachment** 候補。Reservation の `reservation_site_url` はリンクのみ — バイナリ正本は持たない。

---

## 12. 改善候補（v1.19.0 では実装しない）

優先度は **提案のみ**。実装は Hardening 第2弾以降で判断。

| # | 候補 | 種別 | 備考 |
|---|---|---|---|
| 1 | `trip doctor` — confirmation 欠落、期間逆転 | optional warning | Implementation Plan §9 と整合 |
| 2 | Markdown — Itinerary 節内インライン Reservation | export-md | 集約セクションと併記 |
| 3 | `reservation_site_name` フィールド | MVR 拡張 | Entity Design にあり、v1.18.0 で省略 |
| 4 | contact_* / website | MVR 拡張 | しおりの「連絡先はどこか」 |
| 5 | flight / rental_car `details_json` または export ネスト | 種別拡張 | Entity Design §7–9 |
| 6 | Remark → Reservation 移行 helper | CLI / doctor suggestion | canonical 移行期支援 |
| 7 | `trip diff` — Expense 比較 | diff | Reservation と同型 |
| 8 | canonical 以外の **しおり検証用小規模 Trip** | sample | okinawa と分離 |

---

## 13. v1.19.0 スコープ（本書）

### 実施する

| 項目 | 内容 |
|---|---|
| 仕様書 | 本ドキュメント |
| 索引 | [specifications/README.md](README.md) |
| 参照更新 | travel-ledger-responsibilities、reservation-model 系列 |
| v1 Hardening | 実装後責務の文書化（第一弾） |

### 実施しない

```text
DB / CLI / export / Markdown / diff の実装変更
canonical sample 更新
trip doctor 変更
```

---

## 14. 用語

| 用語 | 意味 |
|---|---|
| **Reservation** | Itinerary 実行に必要な予約・確認情報（構造化 entity） |
| **二層構え** | 保存 Itinerary 配下 / 表示 Trip 集約 |
| **Travel Book** | 旅行しおり — export-md 等による計画共有出力（製品概念） |
| **v1 Hardening** | v1 基盤実装後の責務検証・改善候補の文書化フェーズ |

---

## 15. 実装参照（v1.18.0）

| 領域 | パス |
|---|---|
| CRUD / validation | `src/reservation.rs` |
| Model / export 型 | `src/models.rs` |
| CLI | `src/main.rs` |
| export / import | `src/trip.rs` |
| Markdown | `src/markdown.rs` |
| Diff | `src/diff.rs` |
| 統合テスト | `tests/reservation_cli.rs` |
| サンプル | `samples/reservation_sample_commands.sh` |
