# Checklist Design Memo — Web 版から得た知見

Caglla.Travel **Web 版**で得た Checklist 関連の設計知見です。将来の Checklist 強化や旅行支援機能の検討時に参照するためのメモであり、**v1.x の実装対象ではありません**。

関連: [Travel Support 設計メモ](travel-support-design-memo.md)（旅行支援情報・Destination・注意喚起）/ [Travel Ledger Responsibilities](travel-ledger-responsibilities.md) / [Itinerary モデル](itinerary-model.md)（カテゴリと `checklist-generate`）/ [Export Schema](export-schema.md)

**現行 CLI（参考）:**

- Trip 配下の Checklist CRUD — `src/checklist.rs`
- `trip checklist-generate` — Itinerary カテゴリと組み合わせルールから候補を追加（Provenance 未保持）

---

## 1. 自動生成 Checklist は「多いほど便利」ではなかった

Web 版では Itinerary のカテゴリから Checklist を自動生成していました。

例:

- 国際フライト → パスポート
- ドライブ → 運転免許証
- 海水浴 / プール → 水着、浮き輪

当初は「漏れ防止になるので大量生成した方が便利」と考えていました。

しかし実際には、

> この項目はなぜ生成されたのか？

という疑問をユーザーが持つケースが多く発生しました。

自分で追加した項目ではないため、

> 本当に必要なのか？

という視点から確認が始まってしまい、Checklist そのものへの信頼性が低下する傾向がありました。

---

## 2. 生成理由（Provenance）が重要

後から振り返ると問題は Checklist の内容ではなく、

> なぜ生成されたのか分からない

ことでした。

将来的に自動生成を行う場合は、

- source itinerary
- source category
- generation rule

などを保持できる設計が望ましいと思われます。

表示例:

```text
パスポート
  generated from: International Flight

運転免許証
  generated from: Rental Car
```

または

```text
Remark:
国際フライトが登録されているため生成されました
```

のような説明でも良いかもしれません。

---

## 3. 自動生成は最小限・高信頼を優先した方がよい

Web 版の経験では、

「生成件数が多いこと」

よりも

「誰が見ても必要だと分かる項目だけ生成すること」

の方が価値が高いと感じました。

例えば、

- パスポート
- 運転免許証

のような高信頼項目は有効ですが、

大量の推測ベース項目は逆にノイズになりやすい傾向がありました。

設計原則としては、

> **Generate less, but generate with confidence.**

が適切だと思われます。

---

## 4. 将来の役割拡張（旅行準備支援）

Checklist は将来的に Packing だけでなく Task / Warning / Advisory などの旅行準備支援へ発展する可能性がある。ユーザーが価値を感じたのは持ち物より事前準備・注意喚起である、という知見の詳細は [travel-support-design-memo.md](travel-support-design-memo.md) を参照。

---

## 5. 将来に向けた設計上の学び（自動生成）

自動生成を行う場合の前提として、次が重要である。

| 原則 | 内容 |
|---|---|
| 最小限の自動生成 | 件数より信頼性を優先する |
| 高信頼項目を優先 | 推測ベースの大量生成はノイズになりやすい |
| 生成理由を説明可能にする | Provenance（source itinerary / category / rule）を保持・表示 |
| 経緯を追跡できる | ユーザーが「なぜこの項目か」を確認できる |

**v1.x:** 上記は設計判断の参考資料のみ。現行 `checklist-generate` の挙動変更や Provenance フィールド追加は本メモのスコープ外。
