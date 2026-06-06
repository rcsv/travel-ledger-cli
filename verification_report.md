# Caglla.Travel CLI 改善確認レポート

対象バージョン: v0.6.1 (trip doctor), v0.7.0 (checklist-generate)  
生成日: 2026-06-06

## 実行したコマンド

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
cargo build

bash samples/trip_doctor/generate_outputs.sh
bash samples/checklist_generate/generate_outputs.sh
```

Markdown Export 確認（checklist-generate スクリプト内で実行）:

```bash
cargo run -- trip export-md 1
cargo run -- trip export-md 1 --output samples/checklist_generate/export_after_generate.md
```

## 品質確認結果

| コマンド | 結果 | 詳細 |
|---|---|---|
| `cargo fmt` | 成功 (exit 0) | 整形差分なし |
| `cargo clippy -- -D warnings` | 成功 (exit 0) | warning なし |
| `cargo test` | 成功 (exit 0) | **89 passed**, 0 failed |
| `cargo build` | 成功 (exit 0) | `caglla-cli v0.7.0` |

## v0.6.1: trip doctor 改善確認結果

再生成出力: [`samples/trip_doctor/outputs/`](samples/trip_doctor/outputs/)  
確認サマリー: [`samples/trip_doctor/doctor_verification_summary.md`](samples/trip_doctor/doctor_verification_summary.md)

| 観点 | 結果 |
|---|---|
| itinerary 0件が Info 扱い | OK — `02-empty-itinerary.txt` に `Info` / `No itinerary found.` |
| warnings / suggestions / info 分類 | OK — 各シナリオで期待セクションのみ表示 |
| duration 未設定の件数表示 | OK — `06-missing-duration.txt` に `1 itinerary has no duration estimate` |
| overloaded day 検出 | OK — `03-overloaded-day.txt` |
| restaurant 不足検出 | OK — `04-no-restaurant.txt` |
| travel time 180分以上検出 | OK — `05-high-travel-time.txt` (`3h25m`) |
| combined issues 同時表示 | OK — `07-combined-issues.txt` |

## v0.7.0: checklist-generate 強化確認結果

再生成出力: [`samples/checklist_generate/outputs/`](samples/checklist_generate/outputs/)  
確認サマリー: [`samples/checklist_generate/checklist_generate_verification_summary.md`](samples/checklist_generate/checklist_generate_verification_summary.md)

| 観点 | 結果 |
|---|---|
| 単独カテゴリ default_checklist | OK — 全4シナリオで flight/hotel/beach 等の default が生成 |
| 組み合わせルール checklist | OK — flight+hotel, flight+transport, beach+activity, museum+activity すべて確認 |
| title 重複なし | OK — checklist list に同一 title なし |
| 2回目 generate は skip | OK — 全シナリオで run2 は追加 0 件 |
| sort_order | OK — 例: flight+hotel で ID 1〜8 が連番 |

### 組み合わせ別ハイライト

| シナリオ | 追加件数 (run1) | 組み合わせ由来の主な追加 |
|---|---:|---|
| flight + hotel | 8 | 身分証明書, 充電器（宿泊予約確認は dedup skip） |
| flight + transport | 8 | ETCカード, 運転免許証, レンタカー予約確認 |
| beach + activity | 10 | サンダル, 着替え, 防水バッグ, 酔い止め |
| museum + activity | 7 | 事前予約確認, 入場チケット |

## Markdown Export への影響確認結果

出力:

- [`samples/checklist_generate/outputs/05-beach-activity-export-md.txt`](samples/checklist_generate/outputs/05-beach-activity-export-md.txt)
- [`samples/checklist_generate/export_after_generate.md`](samples/checklist_generate/export_after_generate.md)

| 観点 | 結果 |
|---|---|
| 自動生成 checklist が Checklist セクションに出力 | OK — 10件すべて `- [ ]` 形式で表示 |
| 重複項目なし | OK |
| 既存 Markdown 整形 | OK — Overview / Day / 空行 / Category 箇条書きは維持 |

## 気づいた問題

特になし。確認范围内では v0.6.1 / v0.7.0 の改善意図どおりに動作している。

## 修正が必要そうな点

現時点ではなし。

補足（仕様として許容）:

- `flight + hotel` では `宿泊予約確認` が hotel default で追加された後、combination rule 側では skip として記録される。これは重複排除仕様どおり。
- `beach + activity` でも beach rule 実行時に default 由来の `水着/タオル/日焼け止め` が skip ログに出るが、checklist への重複追加は発生しない。

## 次に確認すべき点

- より大きなサンプル旅行（`samples/markdown_sample_commands.sh`）に `checklist-generate` を適用した場合の件数・出力
- `trip doctor` と `checklist-generate` を同一旅行データで連続実行したときの UX
- v0.5.0 以降の Markdown Overview と doctor / checklist 出力を1つのしおりとして読む体験
