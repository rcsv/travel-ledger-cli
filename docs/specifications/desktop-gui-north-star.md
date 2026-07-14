# Caglla.Travel Desktop GUI North Star

| 項目 | 内容 |
|---|---|
| 位置づけ | Desktop GUI の最終的な製品像・優先順位判断基準（設計構想） |
| 状態 | **documentation-only** — 実装・schema・version bump の許可ではない |
| 前提 | [v4.9.0 Desktop transition](v4.9.0-desktop-transition-and-trip-metadata-foundation.md) / [v4.10.0 read-only vertical slice](v4.10.0-read-only-desktop-vertical-slice.md)（released） |
| 関連 | [travel-support-design-memo.md](travel-support-design-memo.md) / [participant-model.md](participant-model.md) / [v4.7.3 Proposal Fragment concept](v4.7.3-proposal-fragment-concept-spec.md) |

---

## 1. 本書の位置づけ

本書は、Caglla.Travel Desktop が最終的に目指す GUI のユーザー体験を定義するための設計構想である。

現行実装の制約や実現難易度から画面を積み上げるのではなく、最初に「最高の旅行計画体験」を定め、その完成像から現在実装すべき機能を逆算する。

本書は以下を目的とする。

* GUI の最終的な製品像を定める
* 個別機能の優先順位を判断する基準を作る
* DB やサービス層が未成熟なため時期尚早な機能を識別する
* 実装都合による妥協が、将来の画面構造を固定してしまうことを防ぐ
* GUI、CLI、データモデル、Proposal / Fragment、export / import の責務を混同しない

本書は設計構想であり、以下を許可するものではない。

* GUI 実装
* DB schema 変更
* migration
* Profile / Person モデル導入
* Proposal / Fragment 保存方式の確定
* version bump
* commit、push、tag、formal release
* 大規模リファクタリング

---

## 2. Product Promise

Caglla.Travel Desktop は、旅行データの CRUD 管理画面ではない。

> 旅行全体を一望しながら、思いついた予定を置き、並べ替え、詳細を少しずつ育てていける旅行計画の作業台

ユーザーに DB の構造を操作させるのではなく、旅行を組み立ててもらう。

中心となる体験は次の通りである。

```text
旅行を作る
  ↓
日程全体を眺める
  ↓
予定を置く
  ↓
順序や日付を調整する
  ↓
予約・費用・メモなどの詳細を育てる
  ↓
準備状況を確認する
```

---

## 3. 基本設計原則

### 3.1 完成像から逆算する

「現在 DB に何があるか」だけで GUI を決めない。

理想の操作を先に定め、その操作に対して次を判定する。

1. 現行モデルで実装可能
2. write use case の整備が必要
3. DB / domain model が未成熟
4. 外部情報基盤が必要
5. 将来構想としてのみ保持する

### 3.2 文脈を失わせない

Trip、Day、Itinerary を編集するたびに別ページへ移動させない。

選択対象は変わっても、旅行全体の作業場所は維持する。

### 3.3 空欄を責めない

旅行計画は徐々に完成する。

未入力項目はエラーではなく、まだ決めていない余白として扱う。

不要な `Not set` 表示を並べず、存在する情報を中心に見せる。

### 3.4 Quick Capture と詳細編集を分ける

予定の追加時に、category、location、duration、reservation、expense などをすべて入力させない。

最初はタイトルだけでも追加でき、詳細は後から Inspector で育てられるようにする。

### 3.5 DB エンティティをそのままメニューにしない

次のようなトップメニュー構成は避ける。

```text
Trip
Day
Itinerary
Note
Expense
Reservation
```

これは Entity Explorer であり、旅行アプリではない。

GUI はユーザーの目的で構成する。

### 3.6 Local-first を価値として見せる

* ログイン不要
* オフラインでも基本操作が可能
* ユーザー自身が DB ファイルを管理できる
* 保存先が分かる
* アプリが勝手に DB を削除しない
* 保存状態が分かる

local-first は offline-only を意味しない。

外部情報や AI を利用しても、正式な旅行データと採用判断はユーザーの手元で管理する。

---

## 4. 画面全体構造

完成形では、次の3領域を基本とする。

```text
┌ Trip Navigator ─┬──────────── Trip Workspace ────────────┬ Inspector ─────┐
│                 │                                         │                │
│ In progress     │  Overview / Plan / Checklist /          │ 選択中の       │
│ Upcoming        │  Travelers / Money                      │ Trip / Day /    │
│ Draft           │                                         │ Itinerary 詳細  │
│ Past            │                                         │                │
└─────────────────┴─────────────────────────────────────────┴────────────────┘
```

### 左側

Trip Navigator

### 中央

Trip 全体の作業領域

### 右側

選択中の対象を詳細表示・編集する Inspector

---

## 5. Trip Navigator

Trip Navigator は旅行を見つけ、切り替える場所である。

表示順は、現在のユーザー関心に近い順とする。

```text
In progress
Upcoming
Draft
Past
```

### 5.1 In progress

今日が Trip の開始日から終了日の範囲内にある旅行。

```text
start_date <= today <= end_date
```

旅行中は最も関心が高いため、最上部に表示する。

表示例:

```text
In progress

● Okinawa 2026
  Day 2 of 4 · Today
```

### 5.2 Upcoming

開始日が未来の旅行。

開始日が近い順に表示する。

### 5.3 Draft

ユーザーが明示的に Draft とした旅行。

入力項目数や Itinerary 数から自動判定しない。

Draft の導入には明示的な状態管理が必要なため、現行 schema に直ちに追加するとは限らない。

### 5.4 Past

終了日が過去の旅行。

終了日が新しい順に表示する。

件数が増えるため、折りたたみや検索を検討する。

### 5.5 導出可能な状態は保存しない

In progress、Upcoming、Past は日付から導出する。

日付の経過に応じて自然に分類が変わるため、DB の status として保存しない。

明示的な保存状態が必要になる可能性があるのは Draft のみである。

---

## 6. Trip Workspace

Trip を選択すると、中央領域に Trip Workspace を表示する。

完成形の主な作業領域は次の通りとする。

```text
Overview
Plan
Checklist
Travelers
Money
```

これらは DB テーブル名ではなく、ユーザーの目的を表す。

---

## 7. Overview

Overview は Trip 全体の状態を短時間で理解する場所である。

表示候補:

* Trip 名
* 日程
* Main Destination
* Country
* Default Currency
* Summary
* Travelers 数
* Checklist 進捗
* Planned / Actual Money 概要
* Pending Suggestions
* Travel Essentials

情報を詰め込むのではなく、必要な作業領域への入口として機能させる。

---

## 8. Plan

Plan は Desktop の中心機能である。

Day ごとの Itinerary を sequence-first で表示する。

```text
Day 2 · Apr 27

09:00  美ら海水族館
       ↓ 30 min

12:00  海邦丸

＋ Add itinerary
＋ Add from suggestions
```

### 8.1 並び順

* `sort_order` を主とする
* `start_time` は任意の時間ラベルとする
* 時刻未定の予定も自然に扱う
* 空き時間を異常扱いしない

### 8.2 Itinerary の追加

基本操作はその場で行う。

```text
＋ Add itinerary
```

クリック後は、まず title だけでも追加できる。

詳細は Inspector で後から編集する。

### 8.3 並べ替え

完成形では次を目指す。

* 同じ Day 内でのドラッグ移動
* 別 Day へのドラッグ移動
* キーボードによる上下移動
* Undo

GUI が直接 SQL を操作するのではなく、reorder / move の安全な write use case を経由する。

### 8.4 保存

理想は autosave である。

ただし、入力のたびに直接 UPDATE することを意味しない。

以下の契約が必要である。

* 編集確定
* 保存成功
* 保存失敗時の復元
* 画面選択状態の維持
* Undo
* 必要に応じた再読み込み

---

## 9. Inspector

Inspector は選択中の Trip、Day、Itinerary の詳細を表示・編集する。

画面遷移せず、旅行全体を見ながら詳細を育てられることを重視する。

### Itinerary Inspector の表示候補

* title
* start time
* duration
* travel time
* category
* location
* primary Venue
* note
* reservation
* estimate
* expense
* related suggestions

Reservation、Estimate、Expense などは常時すべて展開せず、必要に応じて開く。

---

## 10. Checklist

Checklist は Trip 直下の独立した作業領域とする。

Day や Itinerary の付属情報として扱わない。

### 10.1 Trip ヘッダー

進捗のみを簡潔に表示する。

```text
Checklist 18 / 22
```

クリックすると簡易表示または Checklist ワークスペースへ移動する。

### 10.2 Checklist ワークスペース

```text
Checklist

Travel documents                     3 / 4
☑ Check passport expiration
☑ Confirm entry requirements
☐ Register Visit Japan Web
☑ Buy travel insurance

Packing                              5 / 8
☑ Phone charger
☐ Power adapter
☐ Rain jacket
```

主な操作:

* 項目追加
* 編集
* 削除
* 並べ替え
* check / uncheck
* カテゴリ別表示
* 未完了のみ表示
* 自動生成候補の確認

---

## 11. Travelers

内部ドメイン名は Participant を維持する。

GUI 上の名称は `Travelers`、日本語では「参加者」を基本とする。

「同行者」は自分を除く意味に受け取られやすいため、主名称には使わない。

### 11.1 Travelers ワークスペース

```text
Travelers

★ Tomohiro Awane        You
  Wife
  Father
  Mother
  Son

＋ Add traveler
```

Trip ヘッダーには人数を表示する。

```text
5 travelers
```

### 11.2 Participant と Person の境界

現行 Participant は、その Trip に誰が参加するかを表す Trip-scoped participation record である。

以下の人物情報は、将来の Person / Traveler Profile の責務とする。

* legal name
* passport country
* passport expiry
* date of birth
* mileage program
* contact information
* allergy / care notes
* emergency contact

Trip 新規作成時に Travelers 入力を必須にはしない。

Trip 作成後に追加できることを優先する。

---

## 12. Money

Money は Trip 内の金銭情報を目的別にまとめる作業領域とする。

表示候補:

* Planned total
* Actual total
* Difference
* Estimate
* Expense
* Receipt Inbox
* Participant ごとの支払い情報
* 将来の Settlement

Money は Itinerary 配下の Expense / Estimate の正本構造を壊さず、GUI 上で Trip 全体を集約表示する。

Settlement 未実装の段階で、割り勘結果を推測表示しない。

---

## 13. Travel Essentials

Trip 新規作成時に入力した Country、Main Destination、Dates を利用して、旅行に必要な基本情報を表示する。

ユーザーが入力する主な情報:

* Trip name
* Dates
* Country
* Main Destination
* Default Currency、任意

そこからアプリが導出する情報:

* 入国・渡航条件への導線
* その時期の一般的な天候
* コンセントタイプ
* 電圧
* 周波数
* 通貨
* 安全情報への導線

### 13.1 表示場所

Trip 作成画面の補助パネル、および Overview 内の `Travel Essentials` に表示する。

```text
Travel Essentials

Entry
Electronic travel authorization may be required

Typical weather
Warm, occasional showers

Power
Type A / B · 120 V · 60 Hz

Currency
USD
```

### 13.2 Travel Ledger に保存する情報

* Country
* Main Destination
* Dates
* Default Currency
* ユーザーが作成・採用した Checklist 項目
* ユーザー自身の Note

### 13.3 Travel Ledger に原則保存しない情報

* 現在のビザ要件
* 現在の安全情報
* 一般的な気候データ
* コンセント規格の参照データ
* 外部サイトの一時的な回答

これらは外部参照情報であり、Travel Ledger の正式データとは分離する。

### 13.4 入国条件

入国条件は渡航先だけでは確定しない。

以下に依存する。

* passport country
* nationality
* travel purpose
* duration
* transit country
* participant ごとの条件
* passport expiry

初期段階で `Visa not required` と断定しない。

表示例:

```text
Entry requirements

Passport: Japan
Destination: United States

Electronic travel authorization may be required.

Check official requirements
Last checked: 2026-07-14
```

外部情報を確認したユーザーが、必要な行動を Checklist に追加できるようにする。

```text
Add to Checklist
「ESTA を確認する」
```

外部情報そのものではなく、ユーザーが採用した行動を旅行データにする。

### 13.5 一般的な天候

Country 単位では粗すぎるため、Main Destination の位置と旅行月を使う。

表示は天気予報ではなく、一般的傾向であることを明示する。

```text
Typical weather
一般的な気候傾向
```

---

## 14. Proposal Fragment

### 14.1 基本方針

Fragment は GUI の主役にしない。

Fragment は、外部で生まれた未採用候補を、既存 Trip へ安全に反映するための境界機能である。

```text
外部
  AI
  Web
  Email
  Provider
  Manual input
    ↓
Proposal Fragment
    ↓
Review / Dry-run / Adoption Gate
    ↓
Travel Ledger の正式データ
```

### 14.2 Fragment の位置づけ

* Fragment は小さい Trip ではない
* 採用前は正式な Trip データではない
* schema v8 の外側に存在する
* target は Trip / Day / Itinerary / unresolved を取り得る
* intent は add / update / enrich / delete / reorder 等を取り得る
* 人間の確認なしに自動適用しない

### 14.3 Proposal Inbox

Fragment はローカルアプリのデータ保持方式に左右されるため、完成形では `Proposal Inbox` または `Suggestions Inbox` を持つ。

GUI の一般名称は `Suggestions` を第一候補とする。

Fragment は内部設計用語として維持してよい。

```text
Suggestions

Unassigned                         3
For Okinawa 2026                   2
For Hawaii 2027                    1

瀬底ビーチ サンセット散策
Target: Okinawa 2026 / Day unresolved

美ら海水族館 予約情報
Target: Okinawa 2026 / Itinerary
```

Inbox の責務:

* Fragment の受信
* 保存
* 一覧
* 検索
* 出所表示
* valid_until / stale warning
* target 未確定の保持
* defer
* reject
* applied history

### 14.4 文脈付き入口

Inbox を主役にせず、Trip Workspace 内から利用できるようにする。

#### Trip Overview

```text
Suggestions
3 pending
```

Trip 全体の注意、Note、Checklist 候補などを表示する。

#### Day

```text
＋ Add from suggestions
```

現在の Trip / Day を apply target の候補として事前設定する。

#### Itinerary Inspector

```text
Suggested updates
2 available
```

選択中の Itinerary に対する enrich、reservation、estimate、note などの候補を表示する。

### 14.5 Apply フロー

クリックだけで即時反映しない。

```text
Suggestion を選択
  ↓
Target と placement を確認
  ↓
Dry-run preview
  ↓
Warnings / conflicts / required decisions を表示
  ↓
明示的に Apply
```

GUI 上の操作は、内部的に dry-run と confirm を分離する。

### 14.6 段階的導入

#### Stage 1

```text
Day / Itinerary
  ↓
Open Fragment file または Paste Fragment
  ↓
Validate
  ↓
Dry-run preview
  ↓
Apply
```

Fragment を永続保存しない最小導入。

#### Stage 2

`Save for later` を追加し、ローカル保存を導入する。

#### Stage 3

Suggestions Inbox を導入する。

#### Stage 4

外部連携を導入する。

* AI から送信
* ブラウザ拡張
* Email 取り込み
* OS share menu
* provider integration

Fragment は主役ではなく、「手入力以外から正式データへ入る安全な入口」として扱う。

---

## 15. Trip 新規作成の Golden Workflow

最初の完成体験は次を目指す。

```text
New Trip
  ↓
Trip name / Dates / Country / Main Destination を入力
  ↓
Day を自動生成
  ↓
Trip Workspace を開く
  ↓
最初の Itinerary を追加
```

最初から次を必須入力にしない。

* Travelers
* Reservation
* Expense
* Checklist
* Summary
* 詳細な location
* Venue
* passport information
* travel profile
* budget

30秒程度で Trip を作り、旅行計画を始められることを優先する。

---

## 16. Plan / Guide / Review

同じ旅行データに対し、将来的に3つの利用モードを想定する。

### Plan

旅行前に予定を作り、並べ、準備する。

Desktop 初期の中心。

### Guide

旅行中に次の予定、予約、移動、注意事項を確認する。

モバイルとの相性が強い。

### Review

旅行後に費用、実績、写真、記録を振り返る。

Actual time、Photo、Attachment などの成熟が必要。

初期 Desktop は Plan に集中する。

将来構想を理由に、現在の旅行計画体験を後回しにしない。

---

## 17. 実現可能性マトリクス

### 17.1 現行モデルで実装可能

* Trip Navigator
* Trip Overview
* Day timeline
* Trip 新規作成
* Trip 編集
* Itinerary quick add
* Itinerary 編集
* Itinerary reorder
* Itinerary Day 間移動
* Summary 編集
* Note 編集
* Checklist GUI
* Travelers GUI
* Estimate / Expense / Receipt の表示
* Main Destination / Country / Default Currency
* Fragment file / paste preview
* Fragment dry-run / confirm UI

### 17.2 write use case の整備が必要

* autosave
* Undo
* drag and drop
* optimistic UI
* 保存失敗時の復元
* 複数項目の一括変更
* Trip 作成直後の自動選択
* GUI 向けの stable service facade

### 17.3 データモデル成熟待ち

* Venue の正式な再利用
* Person / Traveler Profile
* パスポート情報の Trip 横断利用
* Participant ごとの入国条件
* Settlement
* Photo
* Attachment
* 複数端末同期
* cloud conflict resolution

### 17.4 外部情報基盤が必要

* 一般的な気候データ
* 現在の渡航条件
* 安全情報
* AI suggestion generation
* provider Fragment search
* Web / Email / browser integration

---

## 18. 機能実装の判断基準

新しい GUI 機能は、次の Gate を順に確認する。

### Gate 1: Product Promise に寄与するか

旅行を一望しながら組み立てる体験を改善するか。

CLI 機能を単に画面へ移しただけではないか。

### Gate 2: 既存モデルで意味が通るか

GUI 都合だけで schema を変更しない。

現行モデルで十分な価値を提供できるなら、まずそれで実装する。

### Gate 3: 安全な write use case があるか

GUI が SQL や CLI 引数を直接組み立てない。

型のある application use case を利用する。

### Gate 4: 失敗時にユーザーを迷子にしないか

* 入力が消えない
* 選択が飛ばない
* 画面全体を初期化しない
* 保存結果が分かる
* 復旧方法が分かる

### Gate 5: 単独で価値を提供できるか

将来機能が完成しなければ価値がない機能を先行しない。

そのリリース単独で、旅行計画が一段便利になることを条件とする。

### Gate 6: 主役と補助機能を混同していないか

主役:

* Trip
* Day
* Itinerary
* Checklist
* Travelers
* Money

補助:

* Proposal / Fragment
* export / import
* Travel Essentials
* external AI / Web information
* Settings

補助機能の完成度を上げるために、旅行計画そのものを後回しにしない。

---

## 19. 推奨する次の検討順序

本書は実装指示ではないが、今後の設計検討は次の順が自然である。

1. Trip Workspace の情報設計
2. Trip 新規作成 Golden Workflow
3. Itinerary quick add
4. Inspector による詳細編集
5. reorder / move の直接操作
6. Checklist ワークスペース
7. Travelers ワークスペース
8. Money ワークスペース
9. Travel Essentials
10. Fragment file / paste review
11. Proposal Inbox
12. 外部 AI / Web 連携

Proposal Inbox、Profile、recent DB 複数件、bundle polish などは有用だが、旅行計画の中心体験より先に主役化しない。

---

## 20. North Star Summary

Caglla.Travel Desktop の完成像は、次の一文に集約する。

> 自分の旅行データを手元に保ちながら、旅行全体を一望し、予定、参加者、準備、費用、外部からの提案を、安全に一つの旅行計画へ育てていけるデスクトップアプリ

Fragment は主役ではない。

Travel Essentials も主役ではない。

export / import も主役ではない。

中心にあるのは常に、

```text
Trip
  ↓
Day
  ↓
Itinerary
```

を人間が自然に組み立てられる体験である。
