# trip advisor 検証サマリー

検証日: 2026-06-06  
バージョン: v0.9.0（Structured DoctorIssue Targets）  
再生成:

```bash
bash samples/trip_doctor/generate_outputs.sh
bash samples/advisor/generate_outputs.sh
bash samples/advisor/generate_outputs_with_commands.sh
```

## 品質確認

- cargo fmt: OK
- cargo clippy: OK（`-D warnings`）
- cargo test: OK（107 passed, 0 failed）
- cargo build: OK

## v0.9.0 変更点

- `DoctorIssueTarget`（Trip / Day / Itinerary）を追加
- MissingDuration は itinerary 単位の issue（advisor は `Itinerary N has no duration estimate`）
- doctor 表示は MissingDuration を件数集約（従来互換）
- `--with-commands` の MissingDuration は `itinerary update <id> --duration 60`

## サンプル確認

| 観点 | Result |
|---|---|
| doctor 表示が大きく崩れていない | OK |
| advisor MissingDuration が itinerary ID 付き | OK |
| `--with-commands` MissingDuration が update コマンド | OK |
| NoRestaurant の `--day N` が target day 由来 | OK |
| clean trip に Try なし | OK |
| combined issues で複数 issue + Try | OK |

## 総合判断

v0.9.0 の Structured DoctorIssue Targets は意図通り: **OK**
