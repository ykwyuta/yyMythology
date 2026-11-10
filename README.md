# 神話大戦 / YYMythology

剣と魔法と召喚術でユーラシアの覇権を競うリアルタイムストラテジー。日本とエジプトを含む地図で、最後の一国になると勝利する。地図上の勢力は架空の時代に共存する。

## 起動

Windows 版

```powershell
cargo run --release --locked --bin yymythology-win
```

ブラウザ版

```sh
bash build.sh
python -m http.server 8000 --directory dist
```

ビルド後に [ローカルのゲーム画面](http://localhost:8000/) を開く。

iPhone・iPadでは横向きでプレイする。タップで選択、長押しや画面上のボタンで命令、ピンチで地図を拡大・縮小できる。神名辞典は画面右上のボタンから開く。

## 召喚と魔法

- **神名辞典（F3）**から19の神話圏、57の神々・天使を確認できる。自国領内の民を選んで辞典を開き、自国の神話圏から選ぶ。民1人を生贄に捧げ、糧180・木90・鉄90を使って都に召喚する。同じ神話を共有する全勢力で同時に3体まで。同じ神格の再召喚には、先に召喚された個体が倒れる必要がある。
- ギリシャではゼウスたち、イランではマルドゥークたち、エジプトではラーたち、日本では天照大神たちが登場する。キリスト教・イスラムの勢力はそれぞれの天使を召喚する。
- **英霊（H）**は、自国領内の民を選んで生贄に捧げる。既定で有効で、タイトル画面で切り替えられる。
- **魔法使い**は兵営で訓練する。選択中に **Z** でファイヤーボール（詠唱1.5秒）、**X** でメテオ（詠唱4秒）。射程内の敵の位置を狙い、着弾時に範囲内の交戦相手にダメージを与える。移動・停止で詠唱を中断できる。

## Cloudflare への公開

他のYYシリーズと同じ構成で、Workers Static Assets または Pages に `dist/` を配信する。

- **Workers** は `npm run deploy`。公開先の Worker 名は `yymythology`。
- **Pages** はビルドコマンドを `npm run build`、出力ディレクトリを `dist` に設定する。

Web版では追加画像を圧縮した別ファイルとして読み込む。ビルド時に配信ファイルがCloudflareの1ファイル25 MiB制限に収まるかを確認する。

## 出典

YYMongolia のコミット [`1d0faef`](https://github.com/ykwyuta/yyMongolia/commit/1d0faef) からゲーム本体、各種アセット、ビルド構成を引き継いだ。`docs/` は移植元の資料。召喚ユニット・魔法使いと、追加勢力やエジプトなどの建物は専用の画像を使用する。生成元のPNGとプロンプトは `assets/mythology/` に収録。

地図は [Natural Earth v5.1.2](https://github.com/nvkelso/natural-earth-vector/tree/v5.1.2/geojson) のパブリックドメインデータを使用。海岸線の収録範囲をヨーロッパ・アジア・エジプトに広げた。`tools/geoprep.py` に Natural Earth の `ne_50m_admin_0_countries.geojson`、`ne_50m_lakes.geojson`、`ne_50m_rivers_lake_centerlines.geojson` の入ったディレクトリと出力先 `src/geodata.rs` を渡すと再生成できる。
