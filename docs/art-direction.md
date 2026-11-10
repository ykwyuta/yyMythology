# 蒼天のユーラシア — アート指針

1115〜1294年のゲームとして、旧YYEmpireの春秋戦国期の共通人物・戦車・建築・青銅器装飾から刷新した。

## 画像と表示規則

`assets/period/` に画像生成ツール（組み込み image_gen）で制作した原画PNG、実行用の乗算済みBGRA、生成プロンプト全文 `generation.json`、一覧 `catalog.jpg` を保存する。

| 文化圏 | 対象国 |
|---|---|
| 草原 | モンゴル、ナイマン、ケレイト、タタル、メルキト、天山ウイグル、キプチャク |
| 宋・北方中国 | 宋、南宋、金、遼、西遼、西夏 |
| 鎌倉日本 | 日本 |
| 高麗 | 高麗 |
| イスラーム | ホラズム、セルジューク、アッバース朝、ゴール朝、エジプト、ルーム、ブルガール |
| ラテン | エルサレム、ポーランド、ハンガリー |
| ビザンツ・グルジア・ルーシ | ビザンツ、グルジア、キエフ、ウラジーミル、ノヴゴロド |
| 南方 | 大理、大越 |
| チベット | 吐蕃 |
| 北方森林 | 骨嵬 |

文化圏内では絵を共有し、所属国は既存の国色バッジで識別する。城邑の建築は現在の所有国ではなく建設元 `Building.origin` で選ぶ。固有兵は `Unit.ut` を優先し、服属国の兵を雇っても装備が変わらない。船4種、戦象、西夏の潑喜、騎射兵をそれぞれ対応する画像で表示する。英霊46名は人物データの順に独立した図像を持つ。肖像の顔貌は創作的な解釈で、写真や確定した復元図ではない。

`src/art/historical.rs` が両プラットフォーム共通の選択・読み込みを担う。通常・左右反転・戦場の霧の画像を描画デバイスの再生成時にも作り直す。

原画を差し替えた後は Pillow を利用できる環境で `python tools/import_period_art.py` を実行する。スクリプトはグリッド分割、透明領域のトリミング、寸法の正規化、BGRAへの変換、アプリアイコンの書き出しを行う。通常ビルドはPython不要で、BGRAを直接埋め込む。

## UI

濃紺のフェルトと織物を思わせる地、蒼色の選択、羊皮紙色の文字、控えめな金属色の縁。題字は明朝・セリフ体とし、旧来の鼎・戈・朱旗の扁額を馬尾の旗と水平線の飾りへ変更。ブラウザ起動画面・PWA・Windowsアイコンにも同じ方向性を適用する。

## 制作時に参照した資料

- [Royal Armouries: Arms and Armour of the Mongol Empire](https://royalarmouries.org/objects-and-stories/stories/the-mongols/arms-and-armour-of-the-mongol-empire)
- [The Met: Lamellar Shoulder Defenses](https://www.metmuseum.org/art/collection/search/788894)
- [National Museum of Korea: Japan / o-yoroi](https://www.museum.go.kr/ENG/contents/E0201090500.do?relicId=25949&schM=view&showHallId=756)
- [The Met: Fashion in European Armor, 1000–1300](https://www.metmuseum.org/it/essays/fashion-in-european-armor-1000-1300)

装備・建築は小さなRTSスプライトとして判別しやすいよう簡略化している。近世の天守・全身板金甲冑・銃器・近代国旗を避けることを生成条件に含めた。
