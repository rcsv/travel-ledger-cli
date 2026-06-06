# trip advisor 検証サマリー

検証日: 2026-06-06  
バージョン: v0.8.1（`--with-commands` 追加）  
再生成:

```bash
bash samples/advisor/generate_outputs.sh
bash samples/advisor/generate_outputs_with_commands.sh
```

## 品質確認

- cargo fmt: OK（警告・エラーなし）
- cargo clippy: OK（`-D warnings`、警告なし）
- cargo test: OK（101 passed, 0 failed）
- cargo build: OK

## CLI コマンド体系（表記整合）

| 種別 | 形式 | 例 |
|---|---|---|
| Trip 系 | `cargo run -- trip ...` | `trip advisor 1`, `trip doctor 1` |
| Itinerary 系 | `cargo run -- itinerary ...` | `itinerary add`, `itinerary list`, `itinerary timeline` |
| カテゴリ設定 | `itinerary update --category` | add 時点では不可 |

README / samples / `Try` 出力は上記に統一。`trip itinerary ...` 表記はリポジトリ内に存在しない。

## サンプル確認（通常 `trip advisor`）

| Sample | Expected issue | Expected advice | Result |
|---|---|---|---|
| 01-clean-trip | none | no major issues / no advice | OK |
| 02-empty-itinerary | EmptyItinerary | Start by adding at least one itinerary. | OK |
| 03-overloaded-day | OverloadedDay | move activities / leave buffer time | OK |
| 04-no-restaurant | NoRestaurant | add lunch or dinner plan | OK |
| 05-high-travel-time | HighTravelTime | reduce travel time / group nearby attractions | OK |
| 06-missing-duration | MissingDuration | add estimated duration | OK |
| 07-combined-issues | multiple | all corresponding advice | OK |

出力: [`outputs/`](outputs/) — `Try` なし

## `--with-commands` 確認

| Sample | Expected Try | Result |
|---|---|---|
| 01-clean-trip | Try なし | OK |
| 02-empty-itinerary | `itinerary add ... "First activity"` | OK |
| 03-overloaded-day | `itinerary timeline` + `itinerary list` | OK |
| 04-no-restaurant | `itinerary add` + `itinerary update --category restaurant` | OK |
| 05-high-travel-time | `itinerary timeline` + `itinerary list` | OK |
| 06-missing-duration | `itinerary list` | OK |
| 07-combined-issues | 各 issue に Advice + Try（Day 2/3 反映） | OK |

出力: [`outputs_with_commands/`](outputs_with_commands/)

### Try コマンド実行確認

| コマンド | 結果 |
|---|---|
| `itinerary add 1 --day 1 --time 12:00 --duration 60 "Lunch"` | OK |
| `itinerary update 1 --category restaurant` | OK |
| `itinerary timeline 1` | OK |
| `itinerary list 1` | OK |
| `itinerary add ... --category restaurant`（旧 Try 形式） | NG（add は `--category` 非対応）→ 2 段階に修正済み |

## doctor / advisor 整合確認

- doctor と advisor の issue 検出結果: **整合**
- warning 表示の整合: **整合**
- advice 欠落: **なし**
- `--with-commands` は doctor 出力に影響なし

## 総合判断

v0.8.1 の `trip advisor --with-commands` は意図通り動作しているか:

- OK
- 理由: 品質ゲート成功。CLI 表記整合。Try コマンドは手動実行で成功（NoRestaurant は add + update の 2 段階）。
