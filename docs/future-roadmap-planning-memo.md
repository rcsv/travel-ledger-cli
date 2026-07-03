# Future Roadmap Planning Memo — v4.6.x 完了後の方向性

travel-ledger-cli / Caglla.Travel の **次フェーズ以降** を想定した将来機能方針の整理メモです。

**本書の位置付け:**

```text
v4.6.x 現在作業への追加指示ではない
v4.6.x 完了（existing CLI cleanup / service boundary / DTO migration / read-only 表示整理）後の
  次フェーズ候補として扱う planning document
実装指示・スケジュール確約ではない
```

**直近の大前提:** v4.6.x を中断・混在させない。現行作業の正本は [current-work.md](current-work.md)。

関連: [long-term-version-strategy.md](long-term-version-strategy.md)（製品メジャー版の到達イメージ）/ [planning-design-principles.md](specifications/planning-design-principles.md) / [travel-ledger-responsibilities.md](specifications/travel-ledger-responsibilities.md) / [v4.0.0 Travel Book Concept Design](specifications/v4.0.0-travel-book-concept-design.md)

---

## 背景

これまでの開発は、自分たちの旅行体験や設計上の妄想をベースに進めてきた。今後は世の中の旅行アプリ、AI 旅行計画、カレンダー連携、ウォレット的な旅程管理、費用共有などの流れも意識しながら、Travel Ledger としての方向性を整理しておきたい。

**Caglla / travel-ledger-cli は、予約サイト・SNS・業務帳票ツールを目指すものではない。**

基本方針として、以下を **ローカルファーストな旅行データ** として構造化して保持し、必要に応じて外部サービスや人間向け成果物に出力できる **Travel Ledger** を目指す。

```text
旅行案
採用済み旅程
予約
費用
同行者
変更履歴
旅行当日の参照情報
旅行後の振り返り
```

---

## 中核コンセプト — Travel Data Ledger

今後の中核は、単なる「旅行メモ CLI」ではなく、**旅行データの信頼できる台帳** である。

特に重要なのは、AI や外部サービスが作った旅行案をそのまま信用するのではなく、次の流れを持つこと。

```text
1. Proposal として受け取る
2. 比較・確認する
3. 採用したものだけ Trip / Day / Itinerary に昇格する
4. 採用後の旅程を Calendar / Travel Book / Travel Pack などに出力する
```

Caglla は **AI 旅行計画そのもの** になるよりも、AI や人間が作った旅行案を **「成立する旅行データ」として整理・保持する側** に寄せる。

```text
旅行を予約するアプリではなく、
旅行案・予約・費用・同行者・変更履歴・実績を
ローカルで構造化して保持する Travel Data Ledger
```

外部への出口は次に寄せる。

```text
Travel Book
Calendar ICS
Travel Pack
Proposal Envelope
JSON export
```

---

## 将来ロードマップの大枠案

現時点の感覚では、次の流れが自然。**順序は確定ではない。**

```text
v4.6.x
  existing CLI cleanup
  read-only service boundary
  DTO migration
  aggregate / show output 整理

v4.7.x
  Participant stream
  同行者モデル
  年齢層、制約、役割、旅行計画上の考慮事項

v4.8.x
  Issue #66
  Currency ISO validation and XXX rejection
  Money domain hardening

v4.9.x
  Shared Expense / Settlement
  payer / split / allocation / settlement suggestion

v5.x candidate
  Proposal Envelope
  AI / 外部サービス / 手作業で作った旅行案の取り込み
  proposal diff / accept / reject / promote

v5.x candidate
  Calendar export
  ICS export
  Google Calendar / Outlook / Apple Calendar で予定表示

v5.x candidate
  Reservation Evidence / Attachment foundation
  予約証跡、PDF、メール由来、スクリーンショット、URL

v5.x candidate
  Travel Pack export
  旅行当日に使える offline HTML / Markdown / JSON / attachment bundle

v5.x candidate
  Route Segment / Transport Cost
  移動区間、所要時間、距離、高速料金、燃料費、駐車場

v5.x candidate
  Itinerary Change Log
  旅程変更履歴、変更理由、当初案と実績の差分
```

**Participant の後に Money domain を固め、その後 Shared Expense に入る流れ** はかなり自然と考えている（v2 Participant Foundation / v3 Shared Expense は既にリリース済み — 上記 v4.7.x / v4.9.x は **次段階の拡張** を指す）。

> **注:** 本メモの v5.x candidate は [long-term-version-strategy.md](long-term-version-strategy.md) の製品メジャー v5（Travel Journal）等とは **別軸の候補リスト**。v4.6.x 完了後に両ドキュメントの整合を取り直す。

---

## v4.8.x — Issue #66: Currency ISO validation

Participant stream（v4.7.x）の後、Shared Expense（v4.9.x）に入る **前に** Issue #66 “Currency ISO validation and XXX rejection” を扱いたい。

**理由:** 参加者ごとの費用負担や精算を考える前に、Money domain の入力境界を固めておきたい。

**位置づけ:** DB schema や export schema の変更ではなく、**入力 validation の hardening**。

```text
- DB / export representation は TEXT / String のまま維持する
- lowercase input は uppercase に正規化する
  - jpy -> JPY
  - usd -> USD
- ISO 4217 alpha-3 として存在しないコードは拒否する
  - JPN
  - ABC
  - ZZZ
- XXX は ISO 4217 上は存在するが、Caglla の旅行費用として意味がないため拒否する
- DB enum 化や storage migration はしない
- validate_currency_code() を Money domain の境界として強化する
- 可能なら iso_currency crate の利用を検討する
- minor unit lookup の ISO-backed 化は可能なら行うが、scope を広げすぎない
```

---

## Calendar export — 復活候補

過去の caglla.travel では、calendar 配信によって Google Calendar / Outlook に予定を表示できていた。この考え方は travel-ledger-cli でも **復活候補** にしたい。

**初期方針:** Google Calendar API や Microsoft Graph API に直接接続するのではなく、**ICS export から始める**。

```text
trip export-calendar --trip <id> --output trip.ics
```

または既存コマンド体系に合わせて:

```text
trip calendar-export --trip <id> --output trip.ics
```

### 初期判断基準

Calendar event として出すかどうかの判断基準は、**ユーザーにとって直感的であること** を優先する。

```text
start_time / finish_time または duration が入力されている Itinerary は、
原則 Calendar event として出す
```

理由:

```text
時刻を入れた
  ↓
予定として扱ってほしい
  ↓
カレンダーに出る
```

Anchor / Flexible / Optional のような itinerary role による出力制御は、将来的な option として検討してよいが、**初期 default にすると**「時刻を入れたのになぜカレンダーに出ないのか」という違和感が出る可能性がある。

### 段階的ロードマップ

```text
Phase 1:
  ICS export
  time 付き Itinerary を Calendar event として出力

Phase 2:
  --scope timed / anchors / all

Phase 3:
  calendar_visibility: auto / include / exclude

Phase 4:
  Calendar feed / subscription URL

Phase 5:
  Google Calendar / Outlook API direct sync
```

---

## 採用候補 — 将来機能

### Proposal Envelope

AI、旅行会社、ブログ、自分の手入力、家族の案などを、いきなり Trip に入れず、まず **Proposal** として保存する仕組み。

```text
proposal import
proposal list
proposal show
proposal diff
proposal accept
proposal reject
proposal promote
```

- **Trip** = 採用済みの計画
- **Proposal** = まだ候補の旅行案

この分離は重要。

### Constraint / Feasibility Advisor

AI や人間が作った旅程に対して、現実的に成立するか確認する機能。Caglla は旅行計画 AI そのものよりも、**旅行計画の feasibility checker** として強くできる。

例:

```text
- 移動時間が短すぎないか
- 食事時間が抜けていないか
- 高齢者には詰め込みすぎではないか
- チェックイン前の荷物をどうするか
- 空港到着時間がギリギリすぎないか
- レンタカー返却と給油が成立しているか
- 雨天時の代替案があるか
```

### Reservation Evidence / Attachment

予約そのものだけでなく、**予約の証跡** を持てるようにする。

```text
reservation.source_type
  manual
  email
  pdf
  screenshot
  url
  wallet_pass
  imported_json

reservation.evidence_ref
reservation.confirmation_number
reservation.provider_name
reservation.contact_phone
reservation.cancellation_policy
reservation.deadline_at
```

将来の Attachment model と相性がよい。

### Travel Pack

旅行当日にスマホや PC で開ける **offline pack**。Travel Book の拡張として自然。

```text
travel-book.md
travel-book.html
itinerary.json
reservations/
attachments/
emergency.txt
map-links.html
qr.html
```

### Route Segment / Transport Cost

Itinerary だけではなく、**移動区間そのもの** を扱う候補。

```text
route_segment
  from_itinerary_id
  to_itinerary_id
  mode
  distance_km
  duration_minutes
  toll_amount
  fuel_amount
  parking_amount
  buffer_minutes
```

沖縄旅行のように、レンタカー移動・高速道路・給油・駐車場が旅程成立に大きく影響するケースでは価値が高い。

### Itinerary Change Log

旅行計画は一発で完成しない。変更履歴を持てるとよい。

```text
変更前
変更後
理由
誰が変更したか
いつ変更したか
天候 / 体調 / 混雑 / 予算 / 営業時間変更
```

旅行後の振り返りにも効く。

---

## 避けたい方向

### SNS 化

過去の caglla.travel では「いいね」等の SNS 機能も実装したが、機能が複雑になりすぎ、プロダクトの重心がぶれやすかった。

**travel-ledger-cli / Caglla.Travel では SNS 的な方向は避ける。**

```text
- いいね
- フォロー
- コメント
- 通知
- タイムライン
- ランキング
- SNS 的なマイページ
- 公開投稿を中心にした旅行共有
```

Caglla は **人を集める場所ではなく、旅行データを整えて外へ持ち出せる場所**。

```text
Caglla の中で交流させるのではなく、
Caglla から Travel Book / Calendar / Travel Pack / JSON として外に出す
```

### Excel / CSV export を主要機能にしない

Caglla / Travel Ledger のコンセプトがぶれるため、**主要なユーザー向け機能としては採用しない**。

Caglla が作りたいもの:

```text
旅のしおり
Travel Book
旅行台帳
Proposal Envelope
Calendar ICS
Travel Pack
構造化 JSON
```

であって、**業務帳票や表計算前提の経費精算ツールではない**。

```text
Excel / CSV export should not be treated as a primary user-facing feature.

Travel Ledger is not a business reporting tool or spreadsheet-first expense
system. Its primary outputs should remain travel-native artifacts:
structured JSON, Travel Book Markdown/HTML, Calendar ICS, Proposal envelopes,
and offline Travel Packs.

CSV-like outputs may be considered only for debugging, migration, or developer
inspection, but they should not shape the product concept or user-facing
roadmap.
```

表示上、費用一覧や精算結果が **表になること** は問題ない。それを Excel / CSV file export として **主要成果物にする** のは避ける。

### その他 — 慎重または避ける

```text
- 直接予約
- 決済
- OTA 化
- SNS 化
- リアルタイム地図ナビ
- フライト遅延リアルタイム通知
- デジタル ID 管理
- Excel / CSV を主要 export にすること
```

特に予約・決済・返金・在庫・問い合わせ対応は責任が重く、Caglla の中核から外れる。

```text
Caglla は予約する場所ではなく、予約した事実を安全に持つ場所
```

---

## 早見表

| 区分 | 内容 |
|---|---|
| **中核** | ローカルファースト Travel Data Ledger |
| **Proposal フロー** | 受け取る → 比較 → 採用 → 出力 |
| **v4.7.x 候補** | Participant stream 拡張（年齢層・制約・役割） |
| **v4.8.x 候補** | Issue #66 — Money domain validation hardening |
| **v4.9.x 候補** | Shared Expense / Settlement 拡張 |
| **v5.x 候補** | Proposal Envelope, Calendar ICS, Evidence, Travel Pack, Route Segment, Change Log |
| **主要出力** | Travel Book, Calendar ICS, Travel Pack, Proposal Envelope, JSON |
| **避ける** | SNS, OTA, 決済, Excel/CSV 主要 export |

---

## 改訂

本メモはプロダクト判断に応じて更新する。v4.6.x 進行中は [current-work.md](current-work.md) を正本とし、本書は **着手判断まで参照しない**。
