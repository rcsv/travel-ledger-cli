# Release tools

Caglla.Travel CLI の release 作業を補助するためのローカル用スクリプトです。

このディレクトリのスクリプトは、release 時の手入力を減らし、tag / GitHub Release title / release notes / assets 確認の表記ゆれを防ぐことを目的とします。

```text
tools/release/
  env.sh            # 共通設定・共通関数
  full-release.sh   # create-tag → release を一括実行（推奨）
  create-tag.sh     # make check → commit → tag → push
  release.sh        # GitHub Release の作成/更新 + assets確認
```

## 前提

以下のコマンドが利用できること。

```bash
git
cargo
make
gh
```

GitHub CLI は事前に認証しておく。

```bash
gh auth status
```

未認証の場合:

```bash
gh auth login
```

## 基本的な使い方

release 対象バージョンと release title を **1回だけ** 指定して実行する。

```bash
tools/release/full-release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

`full-release.sh` は `create-tag.sh` → `release.sh` を順に実行する。通常の release はこのコマンドだけで足りる。

上記の場合、release title は次のようになる。

```text
v4.1.2 Okinawa Travel Book sample enrichment
```

release notes は次のファイルを使用する。

```text
docs/releases/v4.1.2-notes.md
```

### 段階的に実行する場合

tag 作成と GitHub Release を分けたいとき、または `create-tag.sh` だけ先に走らせたいときは個別スクリプトを使う。いずれも positional args を受け付ける。

```bash
tools/release/create-tag.sh v4.1.2 "Okinawa Travel Book sample enrichment"
tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

### 環境変数で version / title を渡す

同じ値を繰り返し入力したくない場合、`VNUM` / `VSTR` を export して引数なしで呼べる。

```bash
export VNUM=v4.1.2
export VSTR="Okinawa Travel Book sample enrichment"

tools/release/create-tag.sh
tools/release/release.sh
```

1行で export して連続実行する例:

```bash
VNUM=v4.1.2 VSTR="Okinawa Travel Book sample enrichment" \
  bash -c 'tools/release/create-tag.sh && tools/release/release.sh'
```

`full-release.sh` も同様に環境変数だけで実行できる。

```bash
export VNUM=v4.1.2
export VSTR="Okinawa Travel Book sample enrichment"
tools/release/full-release.sh
```

## full-release.sh

`full-release.sh` は `create-tag.sh` と `release.sh` を同じ version / title で順に実行する。通常の release 導線はこれを使う。

実行例:

```bash
tools/release/full-release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

## create-tag.sh

`create-tag.sh` は、release commit と annotated tag を作成し、`origin/master` と tag を push する。

実行例:

```bash
tools/release/create-tag.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

主な処理:

```text
1. master branch 上であることを確認
2. Cargo.toml の version が指定バージョンと一致することを確認
3. docs/releases/<version>-notes.md が存在することを確認
4. local / remote に同じ tag が存在しないことを確認
5. make check を実行
6. git add .
7. release commit を作成
8. annotated tag を作成
9. origin/master と tag を push
```

作成される commit message:

```text
Release v4.1.2 — Okinawa Travel Book sample enrichment
```

作成される tag message:

```text
v4.1.2 Okinawa Travel Book sample enrichment
```

### make check を省略する場合

通常は推奨しないが、既に直前で確認済みの場合のみ以下で省略できる。

```bash
SKIP_CHECK=1 tools/release/create-tag.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

## release.sh

`release.sh` は、GitHub Release を作成または更新し、Release assets が揃うことを確認する。

実行例:

```bash
tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

主な処理:

```text
1. GitHub Release が自動作成されるのを待つ（任意）
2. 3OS assets が揃うまで待つ
3. assets 揃い後に title / notes を更新（または Release が無ければ create）
4. 最終的な release 情報を JSON で表示
```

v4.6.43 以降、`release.sh` は **assets 待機を title/notes 編集より先** に実行する。Release workflow 実行中の `gh release edit` は macOS asset 欠落などの race を招く可能性があるため、workflow 完了後の編集を推奨する（詳細: [v4.6.43 spec](../../docs/specifications/v4.6.43-release-workflow-asset-upload-follow-up.md)）。

期待する assets:

```text
travel-ledger-cli-<version>-linux-amd64.tar.gz
travel-ledger-cli-<version>-macos-arm64.tar.gz
travel-ledger-cli-<version>-windows-amd64.zip
```

## env.sh

`env.sh` は共通設定・共通関数を定義するファイルです。
直接実行せず、`full-release.sh` / `create-tag.sh` / `release.sh` から読み込まれます。

主な設定:

```bash
REPO=rcsv/travel-ledger-cli
REMOTE=origin
MAIN_BRANCH=master
PACKAGE_NAME=travel-ledger-cli
```

必要に応じて環境変数で上書きできる。

例:

```bash
REPO=rcsv/travel-ledger-cli MAIN_BRANCH=master tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

## 待ち時間の調整

`release.sh` は GitHub Actions による Release 作成と assets upload を待つ。

必要に応じて待ち時間を変更できる。

```bash
WAIT_RELEASE_SECONDS=240 WAIT_ASSETS_SECONDS=600 \
  tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

## 通常の release 手順

1. version / docs / release notes を更新する
2. `make check` を通す
3. release commit / tag / push を作成する
4. GitHub Release を整える
5. assets を確認する

スクリプトを使う場合:

```bash
tools/release/full-release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

段階的に実行する場合:

```bash
tools/release/create-tag.sh v4.1.2 "Okinawa Travel Book sample enrichment"
tools/release/release.sh v4.1.2 "Okinawa Travel Book sample enrichment"
```

最後に確認:

```bash
git status

gh release view v4.1.2 \
  --repo rcsv/travel-ledger-cli \
  --json tagName,name,assets,url
```

期待する状態:

```text
working tree clean
Release title が意図した名前になっている
3OS assets が揃っている
```

## 注意点

* `env.sh` は直接実行しない。
* release title は `<version> <title>` の形で生成される。
* `create-tag.sh` / `release.sh` は positional args がなければ `VNUM` / `VSTR` を読む。
* `create-tag.sh` は working tree に変更がない場合は失敗する。
* 既に local / remote tag が存在する場合は失敗する。
* `release.sh` は tag が push 済みであることを前提とする。
* `release.sh` は **3 OS assets が揃ってから** title / notes を編集する（v4.6.43+）。
* Release workflow 実行中の `gh release edit` は避ける。完了後に `release.sh` または `gh release edit` を使う。
* macOS asset だけ欠けた場合はローカル `cargo build --release` + `gh release upload` で recovery 可能（v4.6.43 spec 参照）。
* 通常は `full-release.sh` を使い、version / title の手入力は1回に留める。


