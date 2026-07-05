# Non-normative examples — Proposal / Fragment / materialize

**Trip Proposal Envelope** と **Proposal Fragment** の **conceptual / draft** example files です。

```text
JSON schema 未確定
validate-export 対象外
authoring guide / narrative 用
```

schema v8 Trip の public examples（validate-export 対象）は [examples/](../examples/) を参照してください。

---

## ファイル一覧

| ファイル | 内容 |
|---|---|
| [trip-proposal-envelope.example.json](trip-proposal-envelope.example.json) | 旅行全体の未採用案 |
| [proposal-fragment.example.json](proposal-fragment.example.json) | 既存 Trip への部分提案 |
| [materialize-before-after.md](materialize-before-after.md) | gate 前後の関係 |

---

## 重要な区別

| | schema v8 Trip | Trip Proposal Envelope | Proposal Fragment |
|---|---|---|---|
| 状態 | 採用済み | 未採用（全体案） | 未採用（部分案） |
| validate-export | **対象** | 対象外 | 対象外 |
| JSON schema | **確定**（export v8） | **未確定** | **未確定** |
| 小さい Trip？ | — | いいえ | **いいえ** |

---

## validate-export

```text
本ディレクトリの JSON → trip validate-export に通さない
Envelope → proposal validate / show / inspect（v4.7.9+）
Fragment → fragment validate（v4.7.11+）
gate 通過後に生成された schema v8 Trip → validate-export 対象
```

---

## 関連

- [examples.md](../examples.md)
- [ai-json-generation-guide.md](../ai-json-generation-guide.md)
- [v4.7.2 Envelope spec](../../specifications/v4.7.2-trip-proposal-envelope-concept-spec.md)
- [v4.7.3 Fragment spec](../../specifications/v4.7.3-proposal-fragment-concept-spec.md)
- [v4.7.8 Implementation planning spec](../../specifications/v4.7.8-proposal-implementation-planning.md)
- [v4.7.4 Materialize gate spec](../../specifications/v4.7.4-materialize-gate-concept-validation-rules.md)
