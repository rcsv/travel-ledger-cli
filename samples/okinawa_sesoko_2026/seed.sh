#!/usr/bin/env bash
# 沖縄・瀬底 2026-04-26〜04-29 canonical sample v1
#
# 使い方（リポジトリルートから）:
#   bash samples/okinawa_sesoko_2026/seed.sh
#
# 元データ: EstimateTrip_20260426.pdf（実旅行の行動台帳）
# v1: v4.1.2 Travel Book fixture（Summary / Note / Reservation / Estimate）

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
WORK="${CAGLLA_SAMPLE_WORKDIR:-$ROOT}"
cd "$WORK"

run() {
  cargo run --quiet --manifest-path "$ROOT/Cargo.toml" -- "$@"
}

exp() {
  local iid=$1 amount=$2
  shift 2
  run expense add --itinerary "$iid" --amount "$amount" --currency JPY "$@"
}

rec() {
  run receipt add --trip 1 "$@"
}

echo "==> DB を初期化"
run db reset

echo "==> Trip を作成"
run trip add "沖縄 瀬底 4日間" --start 2026-04-26 --end 2026-04-29

echo "==> Day 1 (2026-04-26)"

run itinerary add 1 --day 1 --time 06:00 --order 1 --location "粟根家" \
  "出発、せっちゃんやっちゃんピックアップ"
run itinerary update 1 --category transport

run itinerary add 1 --day 1 --time 06:30 --order 2 --location "東浦→セントレア" \
  "高速道路 東浦→セントレア"
run itinerary update 2 --category transport
exp 2 630 --note "費用区分: 旅費 / 領収書: なし / 備考: 東浦→知多半島道路→セントレア" \
  --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 06:50 --order 3 --location "セントレア駐車場" \
  "P1 G Parking"
run itinerary update 3 --category transport
exp 3 13000 --note "費用区分: 旅費 / 領収書: R-007 / 備考: 予約番号 : 655098, 含 予約料 ¥1,000" \
  --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 07:30 --order 4 --location "セントレア空港" \
  "チェックイン"
run itinerary update 4 --category flight

run itinerary add 1 --day 1 --time 07:44 --order 5 --location "セントレア3F" \
  "朝食 スタンダードコーヒー"
run itinerary update 5 --category restaurant
exp 5 4940 --note "費用区分: 食費 / 領収書: R-025" --expense-date 2026-04-26
exp 5 400 --title "ジュース買い足し" \
  --note "費用区分: 食費 / 領収書: R-020 / 備考: ジャスミン茶、おーいお茶" \
  --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 08:30 --order 6 --location "中部国際空港" \
  "NU045 NGO ⇒ OKA (11:00着)"
run itinerary update 6 --category flight
exp 6 210240 --note "費用区分: 旅費" --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 11:30 --order 7 --location "那覇国際空港 食堂" \
  "昼食"
run itinerary update 7 --category restaurant
exp 7 9040 --note "費用区分: 食費 / 領収書: R-003 / 備考: 一人1000円程度" \
  --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 12:30 --order 8 --location "那覇国際空港" \
  --note "スタッフに電話 : +81 80 6489 6688。エレベーターで3階に移動。[出口5番]。" \
  "レンタカー屋へのシャトルバス乗車"
run itinerary update 8 --category transport

run itinerary add 1 --day 1 --time 13:00 --order 9 --location "Ks Rent A Car" \
  --note "要：ETCカード" "Toyota Alphard 又は同等車種"
run itinerary update 9 --category transport
exp 9 51562 --note "費用区分: 旅費 / 備考: 要：ETCカード" --expense-date 2026-04-26

run itinerary add 1 --day 1 --order 10 --location "首里城" "観光:首里城"
run itinerary update 10 --category activity
exp 10 1760 --note "費用区分: 旅費 / 領収書: R-012 / 備考: 大人:¥400, 子供: ¥160" \
  --expense-date 2026-04-26
exp 10 3405 --title "御城印、ロルバーンメモ帳" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘 / 領収書: R-002" --expense-date 2026-04-26
exp 10 600 --title "アイスクリーム" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘 / 領収書: R-026" --expense-date 2026-04-26

run itinerary add 1 --day 1 --order 11 "移動 高速道路"
run itinerary update 11 --category transport
exp 11 1000 --note "費用区分: 旅費 / 備考: 高速道路の大体の費用" --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 16:00 --order 12 --location "ザ・ビッグもとぶ店" \
  "夕食の買い出し"
run itinerary update 12 --category shopping
exp 12 15371 --note "費用区分: 食費 / 領収書: R-016" --expense-date 2026-04-26
exp 12 2110 --title "休息時間とか" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘" --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 16:30 --order 13 --location "瀬底大橋" \
  "瀬底島に入る"
run itinerary update 13 --category transport

run itinerary add 1 --day 1 --time 16:40 --order 14 --location "ヒルトン瀬底" \
  --note "管理費など" "チェックイン"
run itinerary update 14 --category hotel
exp 14 15000 --note "費用区分: 旅費 / 備考: 管理費など" --expense-date 2026-04-26

run itinerary add 1 --day 1 --time 18:00 --order 15 "夕食 部屋"
run itinerary update 15 --category restaurant
exp 15 2110 --title "食材追加 CHIM DON DON" \
  --note "費用区分: 食費 / 領収書: R-004 / 備考: 塩、しょうゆなど" \
  --expense-date 2026-04-26

echo "==> Day 2 (2026-04-27)"

run itinerary add 1 --day 2 --time 07:00 --order 1 --location "ヒルトン瀬底" "出発"
run itinerary update 16 --category transport

run itinerary add 1 --day 2 --time 07:10 --order 2 --location "海洋公園前店" \
  "朝食 ローソン"
run itinerary update 17 --category restaurant
exp 17 1253 --note "費用区分: 食費 / 領収書: R-008 / 備考: ファミリーマート 本部山川もある" \
  --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 07:30 --order 3 --location "美ら海水族館駐車場" \
  --note "手続きを早々に済ませカフェ準備" "水族館の駐車場に到着"
run itinerary update 18 --category transport

run itinerary add 1 --day 2 --time 08:30 --order 4 --location "美ら海水族館" \
  "水族館に入館"
run itinerary update 19 --category museum
exp 19 9430 --note "費用区分: 旅費 / 備考: 大人:¥2180, 子供:¥710" --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 08:40 --order 5 --location "カフェ オーシャンブルー" \
  "軽食＆ジンベイザメ"
run itinerary update 20 --category restaurant
exp 20 7500 --note "費用区分: 食費 / 備考: 館内カフェに先にいく。一人1500円" \
  --expense-date 2026-04-27

run itinerary add 1 --day 2 --order 6 --location "ドリームセンター" "ドリームセンター入館"
run itinerary update 21 --category activity
exp 21 3040 --note "費用区分: 旅費 / 領収書: R-015 / 備考: 大人:¥760, 子供: 無料" \
  --expense-date 2026-04-27
exp 21 1000 --title "ツノガエルくじ" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘 / 領収書: R-014" --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 11:00 --order 7 --location "スナックスコール" \
  "昼食"
run itinerary update 22 --category restaurant
exp 22 7600 --note "費用区分: 食費 / 領収書: R-010 / 備考: カレー中心。" \
  --expense-date 2026-04-27
exp 22 600 --title "昼食追加" \
  --note "費用区分: 食費 / 領収書: R-011 / 備考: もちもちドーナツを追加" \
  --expense-date 2026-04-27

run itinerary add 1 --day 2 --order 8 --location "ドリームセンター入口" "周遊バス乗車"
run itinerary update 23 --category transport
exp 23 1500 --note "費用区分: 旅費 / 領収書: R-027 / 備考: 遊覧車１回券 ¥300x5 子供も一律" \
  --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 13:00 --order 9 --location "古宇利島" "砂浜見学"
run itinerary update 24 --category beach

run itinerary add 1 --day 2 --time 14:15 --order 10 --location "物販ソラハシ" \
  "お土産 古宇利島"
run itinerary update 25 --category shopping
exp 25 2780 --title "お土産" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘 / 領収書: R-032" --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 14:30 --order 11 --location "沖縄ハナサキマルシェ" \
  "スタバる"
run itinerary update 26 --category restaurant
exp 26 7160 --title "スタバ" --paid-by-name "知弘" \
  --note "費用区分: 個別：知弘 / 領収書: R-006 / 備考: 3人ぐらい1000円の飲み物" \
  --expense-date 2026-04-27

run itinerary add 1 --day 2 --time 19:30 --order 12 --location "AMAHAJI" "夕食"
run itinerary update 27 --category restaurant
exp 27 24375 --note "費用区分: 食費 / 領収書: R-019" --expense-date 2026-04-27

echo "==> Day 3 (2026-04-28)"

run itinerary add 1 --day 3 --time 07:30 --order 1 --location "ヒルトン瀬底" \
  "朝食"
run itinerary update 28 --category restaurant

run itinerary add 1 --day 3 --time 08:30 --order 2 --location "本部港フェリーターミナル" \
  "フェリー乗船"
run itinerary update 29 --category transport
exp 29 6270 --note "費用区分: 旅費 / 領収書: R-018" --expense-date 2026-04-28

run itinerary add 1 --day 3 --time 09:00 --order 3 "出港"
run itinerary update 30 --category transport

run itinerary add 1 --day 3 --time 09:30 --order 4 "伊江島着"
run itinerary update 31 --category transport

run itinerary add 1 --day 3 --time 09:45 --order 5 --location "リリー園" "リリー園着"
run itinerary update 32 --category activity

run itinerary add 1 --day 3 --time 10:00 --order 6 --location "リリー園売店" \
  "朝食の焼きそば"
run itinerary update 33 --category restaurant
exp 33 1000 --title "焼きそば" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 備考: ジュースもろもろ" --expense-date 2026-04-28

run itinerary add 1 --day 3 --order 7 --location "ハイビスカス園" "入園料 ハイビスカス園"
run itinerary update 34 --category activity
exp 34 1300 --note "費用区分: 旅費 / 領収書: R-034 / 備考: 子供:100円、大人:300円" \
  --expense-date 2026-04-28

run itinerary add 1 --day 3 --time 14:20 --order 8 --location "ザ・ビッグもとぶ店" \
  "昼食以降の食材調達"
run itinerary update 35 --category shopping
exp 35 9440 --note "費用区分: 食費 / 領収書: R-013" --expense-date 2026-04-28

run itinerary add 1 --day 3 --time 14:30 --order 9 --location "ヒルトン瀬底" "昼食"
run itinerary update 36 --category restaurant

run itinerary add 1 --day 3 --time 16:00 --order 10 --location "ヒルトン瀬底" "海水浴"
run itinerary update 37 --category beach
exp 37 4500 --title "ビーチセット" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 備考: ビーチセット" --expense-date 2026-04-28

echo "==> Day 4 (2026-04-29)"

run itinerary add 1 --day 4 --time 07:30 --order 1 --location "AMAHAJI" "朝食"
run itinerary update 38 --category restaurant
exp 38 16500 --note "費用区分: 食費 / 領収書: R-001 / 備考: 予約受付なし" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 07:50 --order 2 "土産屋さん"
run itinerary update 39 --category shopping
exp 39 3700 --title "土産屋さん" --paid-by-name "節子" \
  --note "費用区分: 個別：節子 / 領収書: R-005" --expense-date 2026-04-29
exp 39 8380 --title "土産屋さん" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-021 / 備考: マスコットクジラ、フェイスマスク、ポストカード" \
  --expense-date 2026-04-29
exp 39 3200 --title "SAGAWA BOX" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-001" --expense-date 2026-04-29
exp 39 286 --title "宅配便" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-001" --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 10:00 --order 3 --location "ヒルトン瀬底" \
  --note "チェックアウトリミット：10:00" "出発"
run itinerary update 40 --category hotel

run itinerary add 1 --day 4 --order 4 --location "御菓子御殿 恩納店" "お土産 御菓子御殿"
run itinerary update 41 --category shopping
exp 41 6707 --title "お土産" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-032 / 備考: 安納芋の紅芋タルト" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 11:00 --order 5 --location "万座毛" "万座毛見学"
run itinerary update 42 --category activity
exp 42 500 --note "費用区分: 旅費 / 備考: 入場料 ¥100/人" --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 11:24 --order 6 --location "万座毛" \
  "昼食 元祖海ブドウ丼"
run itinerary update 43 --category restaurant
exp 43 6760 --note "費用区分: 食費 / 領収書: R-035 / 備考: 一人5000円相当" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 14:00 --order 7 --location "ウミカジテラス" "買い物"
run itinerary update 44 --category shopping

run itinerary add 1 --day 4 --time 15:20 --order 8 --location "りゅうせき SSG" \
  "ガソリン満タン返し"
run itinerary update 45 --category transport
exp 45 5447 --note "費用区分: 旅費 / 領収書: R-030 / 備考: レギュラー 182円/L、29.93L" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 15:30 --order 9 --location "Ks Rent A Car" \
  --note "レンタカー返却:16:00" "レンタカー返却"
run itinerary update 46 --category transport

run itinerary add 1 --day 4 --time 15:45 --order 10 --location "那覇国際空港" "空港入り"
run itinerary update 47 --category transport

run itinerary add 1 --day 4 --time 16:30 --order 11 --location "那覇空港内 ファミマ" \
  "みなとさんへのおみやげ"
run itinerary update 48 --category shopping
exp 48 782 --title "おみやげ" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-017 / 備考: 沖縄そば、さんぴん茶（飲んじゃった）" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 16:38 --order 12 --location "那覇空港内" "ロイズ"
run itinerary update 49 --category shopping

run itinerary add 1 --day 4 --time 16:45 --order 13 --location "那覇空港内 売店" \
  "ラングドシャ：会社のお土産"
run itinerary update 50 --category shopping
exp 50 2160 --title "ラングドシャ" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-029 / 備考: 会社のお土産" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 18:00 --order 14 --location "Royal Host" "夕食"
run itinerary update 51 --category restaurant
exp 51 8052 --note "費用区分: 食費 / 領収書: R-028 / 備考: じいじばあば席" \
  --expense-date 2026-04-29
exp 51 5115 --note "費用区分: 食費 / 領収書: R-031 / 備考: ともひろ、るみ席" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 18:30 --order 15 --location "那覇国際空港" "チェックイン"
run itinerary update 52 --category flight

run itinerary add 1 --day 4 --time 18:39 --order 16 --location "那覇空港 DFS" \
  "那覇空港内 DFS・売店"
run itinerary update 53 --category shopping
exp 53 3900 --title "ホノルルクッキー" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-024" --expense-date 2026-04-29
exp 53 31800 --title "化粧品" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-022" --expense-date 2026-04-29
exp 53 37500 --title "COACH のバッグ" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-023" --expense-date 2026-04-29
exp 53 445 --title "ジュース買い足し" --paid-by-name "知弘" \
  --note "費用区分: 個別:知弘 / 領収書: R-009" --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 20:05 --order 17 --location "中部国際空港" \
  "NU046 OKA ⇒ NGO (22:05着)"
run itinerary update 54 --category flight

run itinerary add 1 --day 4 --time 22:30 --order 18 --location "セントレア駐車場" \
  "P1 G Parking"
run itinerary update 55 --category transport

run itinerary add 1 --day 4 --time 22:45 --order 19 --location "セントレア→東浦" \
  "高速道路 セントレア→東浦"
run itinerary update 56 --category transport
exp 56 630 --note "費用区分: 旅費 / 領収書: なし / 備考: 知多半島道路 セントレア→東浦" \
  --expense-date 2026-04-29

run itinerary add 1 --day 4 --time 23:10 --order 20 --location "シャトレー" \
  --note "有料道路:700円ぐらい？" "やっちゃんせっちゃんおろす"
run itinerary update 57 --category transport

run itinerary add 1 --day 4 --time 23:30 --order 21 --location "自宅" "帰宅"
run itinerary update 58 --category transport

echo "==> Checklist（旅行準備）"
run checklist add 1 "ETCカード"
run checklist add 1 "レンタカー予約確認"
run checklist add 1 "パスポート・身分証"
run checklist add 1 "駐車場予約確認（セントレア P1G）"
run checklist check 2
run checklist check 4

echo "==> Travel Book fixture（v4.1.2 — Summary / Note / Reservation / Estimate）"
# Itinerary ID は現行投入順の参考値。正本は Day + sort_order + title（v4.1.1 plan §4–5）。

run trip update 1 --summary "高齢者2名を含む5名で、沖縄本島北部・瀬底島を拠点に過ごす4日間の家族旅行。レンタカーで美ら海水族館、古宇利島、伊江島、万座毛を巡り、最終日は那覇空港へ戻る。"

run day update 1 1 --summary "中部国際空港から那覇へ移動し、レンタカーを受け取って首里城を観光。夕方に瀬底島へ入り、買い出し後にヒルトン瀬底へチェックインする移動中心の日。"
run day update 1 2 --summary "朝から美ら海水族館を楽しみ、カフェ オーシャンブルーやドリームセンターを回る。午後は古宇利島・ハナサキマルシェ方面に立ち寄り、夜はホテルで夕食。"
run day update 1 3 --summary "フェリーで伊江島へ渡り、リリー園やハイビスカス園を訪れる。午後は瀬底へ戻り、買い出しと海水浴でゆっくり過ごす日。"
run day update 1 4 --summary "朝食後にチェックアウトし、御菓子御殿・万座毛・ウミカジテラスを経由して那覇空港へ向かう。レンタカー返却後、空港で夕食と買い物を済ませて中部国際空港へ戻る。"

run note add --trip 1 --body "高齢者2名を含むため、移動時間と休憩を多めに見る。レンタカー移動を前提にし、空港・ホテル・観光地間の乗り換え負担を減らす。"
run note add --trip 1 --body "部屋食や買い出しを活用し、外食だけに寄せない。天候や体調次第で、古宇利島・海水浴・空港買い物は調整可能とする。"

run note add --trip 1 --day 1 --body "朝が早いため、セントレア到着後の朝食と休憩を確保する。沖縄到着後はレンタカー受け取りに時間がかかる可能性あり。"
run note add --trip 1 --day 2 --body "美ら海水族館は早め到着を前提にする。カフェや植物園は混雑・体調に応じて滞在時間を調整する。"
run note add --trip 1 --day 3 --body "フェリー利用日。出港時刻を優先し、朝の移動に余裕を持つ。午後はホテル周辺で無理をしない構成にする。"
run note add --trip 1 --day 4 --body "チェックアウト後から夜便まで時間が長い。荷物・高齢者の疲労・レンタカー返却時刻を優先して調整する。"

# R1: Day 1 / sort 3 / P1 G Parking (id 3)
run reservation add --itinerary 3 \
  --reservation-type parking \
  --provider "セントレア P1 G Parking" \
  --confirmation 655098 \
  --remark "出発日の駐車場予約"
# R2: Day 1 / sort 6 / NU045 (id 6)
run reservation add --itinerary 6 \
  --reservation-type flight \
  --provider "NU045 NGO ⇒ OKA"
# R3: Day 4 / sort 17 / NU046 (id 54)
run reservation add --itinerary 54 \
  --reservation-type flight \
  --provider "NU046 OKA ⇒ NGO"
# R4: Day 1 / sort 9 / Toyota Alphard (id 9)
run reservation add --itinerary 9 \
  --reservation-type rental_car \
  --provider "Ks Rent A Car" \
  --remark "ETCカード必須"
# R5: Day 1 / sort 14 / チェックイン (id 14)
run reservation add --itinerary 14 \
  --reservation-type hotel \
  --provider "ヒルトン瀬底" \
  --start-at "2026-04-26T16:40" \
  --end-at "2026-04-29T10:00"

run estimate add --itinerary 6 --amount 200000 --currency JPY \
  --title "航空券（往路）"
run estimate add --itinerary 54 --amount 200000 --currency JPY \
  --title "航空券（復路）"
run estimate add --itinerary 9 --amount 50000 --currency JPY \
  --title "レンタカー"
run estimate add --itinerary 3 --amount 13000 --currency JPY \
  --title "駐車場（出発）"
run estimate add --itinerary 14 --amount 120000 --currency JPY \
  --title "宿泊（3泊）"
run estimate add --itinerary 19 --amount 10000 --currency JPY \
  --title "美ら海水族館"
run estimate add --itinerary 29 --amount 6000 --currency JPY \
  --title "フェリー"
run estimate add --itinerary 27 --amount 80000 --currency JPY \
  --title "食費ざっくり"

echo "==> Receipt Inbox（帰宅後に整理する未整理支払い候補）"
# Receipt は Actual ではない。assign 後に作成された Expense だけが Actual に入る。
rec --day 2 --amount 4860 --currency JPY \
  --memo "美ら海水族館ショップ: ぬいぐるみ、キーホルダー"
rec --day 2 --amount 2340 --currency JPY \
  --memo "ハナサキマルシェ: ちんすこう、紅芋タルト"
rec --amount 3180 --currency JPY \
  --memo "道の駅 許田: お土産まとめ買い"
rec --day 4 --amount 5600 --currency JPY \
  --memo "那覇空港売店: 職場向け土産"
rec --day 4 --memo "コンビニのレシート（金額未入力・あとで確認）"
rec --day 3 --amount 1200 --currency JPY \
  --memo "個人的な雑貨購入（旅行共通費にはしない予定）"
run receipt trash 6

echo ""
echo "==> canonical sample v1 の投入が完了しました (Trip ID: 1)"
echo ""
echo "確認コマンド:"
echo "  cargo run -- trip stats 1"
echo "  cargo run -- receipt list --trip 1"
echo "  cargo run -- trip export-md 1"
echo "  cargo run -- trip export 1 --output samples/okinawa_sesoko_2026/trip-export.json"
echo "  cargo run -- trip validate-export samples/okinawa_sesoko_2026/trip-export.json"
