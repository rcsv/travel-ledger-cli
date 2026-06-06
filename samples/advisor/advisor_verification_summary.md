# trip advisor 確認サマリー (v0.8.0)

生成日: 2026-06-06  
再生成: `bash samples/advisor/generate_outputs.sh`

## 01-clean-trip

- **expected purpose**: 問題なし。advisor も issues なし。
- **actual output file**: [`outputs/01-clean-trip.txt`](outputs/01-clean-trip.txt)
- **confirmed points**:
  - `No major issues found.` が表示される
  - `Warning` / `Advice` ブロックは出ない

## 02-empty-itinerary

- **expected purpose**: EmptyItinerary → Info + advice。
- **actual output file**: [`outputs/02-empty-itinerary.txt`](outputs/02-empty-itinerary.txt)
- **confirmed points**:
  - `Info` に `- No itinerary found.`
  - `Advice` に `- Start by adding at least one itinerary.`

## 03-overloaded-day

- **expected purpose**: OverloadedDay → Warning + 2件の advice。
- **actual output file**: [`outputs/03-overloaded-day.txt`](outputs/03-overloaded-day.txt)
- **confirmed points**:
  - `- Day 1 has many itineraries (8)`
  - `- Consider moving some activities to another day.`
  - `- Leave buffer time for delays and rest.`

## 04-no-restaurant

- **expected purpose**: NoRestaurant → Warning + advice。
- **actual output file**: [`outputs/04-no-restaurant.txt`](outputs/04-no-restaurant.txt)
- **confirmed points**:
  - `- Day 1 has no restaurant`
  - `- Consider adding a lunch or dinner plan.`

## 05-high-travel-time

- **expected purpose**: HighTravelTime → Warning + 2件の advice。
- **actual output file**: [`outputs/05-high-travel-time.txt`](outputs/05-high-travel-time.txt)
- **confirmed points**:
  - `- Day 1 has high travel time (3h25m)`
  - `- Consider reducing travel time.`
  - `- Group nearby attractions together.`

## 06-missing-duration

- **expected purpose**: MissingDuration → Warning + 2件の advice。
- **actual output file**: [`outputs/06-missing-duration.txt`](outputs/06-missing-duration.txt)
- **confirmed points**:
  - `- 1 itinerary has no duration estimate`
  - `- Add an estimated duration.`
  - `- Even a rough estimate improves planning quality.`

## 07-combined-issues

- **expected purpose**: 複数 issue ごとに Warning + Advice ペアが表示される。
- **actual output file**: [`outputs/07-combined-issues.txt`](outputs/07-combined-issues.txt)
- **confirmed points**:
  - Day 1: overloaded / no restaurant / high travel time それぞれ advice 付き
  - Day 2 / Day 3: no restaurant + missing duration も個別ブロック
  - doctor の Suggestions 一覧ではなく、issue 単位の Advice 形式

## doctor との関係

| コマンド | 役割 |
|---|---|
| `trip doctor` | 問題検出（Warnings / Suggestions / Info） |
| `trip advisor` | 問題ごとの改善提案（Warning + Advice） |

doctor の既存出力形式は `analyze_trip_issues` 経由でも維持されています。
