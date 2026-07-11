# AI integration

外部 AI ツールと Caglla.Travel CLI がどう連携するかの概念説明です。生成作法の詳細は [public/ai-json-generation-guide.md](public/ai-json-generation-guide.md)、公開契約は [public/proposals.md](public/proposals.md) を参照してください。

## 責務分担

| 役割 | 誰が担うか | 例 |
|---|---|---|
| **提案の生成** | 外部 AI / provider / 人間 | Trip Proposal Envelope、Proposal Fragment |
| **形式の検証** | CLI | `proposal validate`、`fragment validate`、`trip validate-export` |
| **採用判断** | 人間 | gate 前の review、required decisions の解消 |
| **正式データへの反映** | CLI（人間の confirm 後） | `proposal materialize --confirm`、`fragment apply --confirm` |
| **正本の保管** | ローカル SQLite | 採用済み Trip / Day / Itinerary |

AI は **候補案を出す** 側です。採用・検証・DB 反映は CLI と人間の役割です。AI が schema v8 Trip を直接出力しても、それだけでは正式データにはなりません。

## 3 種類の JSON

| 種類 | いつ使う | CLI での入口 |
|---|---|---|
| **schema v8 Trip** | 採用済み・実日付確定・事実確認済み | `trip validate-export`、`trip import` |
| **Trip Proposal Envelope** | 旅行全体の未採用案 | `proposal validate`、`proposal materialize` |
| **Proposal Fragment** | 既存 Trip への部分提案 | `fragment validate`、`fragment apply` |

未確定の提案を schema v8 Trip に無理に入れないことが基本です。詳細な使い分け: [ai-json-generation-guide.md](public/ai-json-generation-guide.md)

## 典型的な流れ

### 旅行全体の案（Envelope）

```text
外部 AI
  → Trip Proposal Envelope JSON を生成
  → 人間が review
  → travel-ledger-cli proposal validate <file>
  → travel-ledger-cli proposal materialize <file> --dry-run
  → 人間が確認
  → travel-ledger-cli proposal materialize <file> --confirm
  → 新しい schema v8 Trip（DB または JSON）
```

### 既存 Trip への部分案（Fragment）

```text
外部 AI（既存 Trip export を文脈に）
  → Proposal Fragment JSON を生成
  → 人間が review
  → travel-ledger-cli fragment validate <file>
  → travel-ledger-cli fragment apply <file> --dry-run --trip <id>
  → 人間が preview / diff を確認
  → travel-ledger-cli fragment apply <file> --confirm --trip <id>
  → 既存 Trip が更新される
```

`--dry-run` は DB を変更しません。`--confirm` は gate 通過後のみ副作用があります。`--dry-run` と `--confirm` は併用できません。

## 外部 AI ツールとの関係

Caglla.Travel CLI は AI サービスを内蔵しません。連携は **ファイルベース** です。

1. AI が Envelope / Fragment /（場合により）schema v8 候補 JSON を出力する
2. 人間がファイルを保存する
3. CLI が validate / dry-run / confirm を実行する
4. 採用結果がローカル DB または export JSON になる

ChatGPT・Claude・自前スクリプトなど、JSON を生成できるツールなら接続先は問いません。重要なのは、**生成物の種類を間違えない** ことと、**confirm 前に dry-run で確認する** ことです。

## validate と apply の違い

| 操作 | 目的 | DB への影響 |
|---|---|---|
| `proposal validate` / `fragment validate` | ファイル形式・必須フィールドの検査 | なし |
| `proposal materialize --dry-run` | 採用後 Trip JSON の preview | なし |
| `fragment apply --dry-run` | 既存 Trip 更新の preview | なし（read-only DB access） |
| `proposal materialize --confirm` | 新規 Trip を DB に保存 | あり |
| `fragment apply --confirm` | 既存 Trip を更新 | あり |

`trip validate-export` は **schema v8 Trip のみ** が対象です。Envelope / Fragment には使いません。

## 読む順序

```text
1. 本ドキュメント（ai.md）           — 概念と責務分担
2. public/proposals.md              — Envelope / Fragment / gate の公開契約
3. public/ai-json-generation-guide.md — 生成 AI 向け作法・プロンプト例
4. public/examples-non-normative/   — 概念例 JSON
5. command-reference.md             — CLI オプションの正本
6. specifications/                  — 実装・内部仕様（開発者向け）
```

## 関連

- [public/proposals.md](public/proposals.md) — proposal / fragment の公開契約
- [public/ai-json-generation-guide.md](public/ai-json-generation-guide.md) — 生成ルールとプロンプト例
- [public/travel-ledger.md](public/travel-ledger.md) — Travel Ledger の位置づけ
- [command-reference.md](command-reference.md) — CLI 全コマンド
- [AGENTS.md](../AGENTS.md) — AI coding agent 向けリポジトリ作業ルール
