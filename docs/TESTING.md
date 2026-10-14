# 検証記録

## 旧.NET版 v0.1.0（2026年10月14日）

- .NET SDK 10.0.402でWindows x64単一exeをクロスビルド：成功
- ロジック・注入列・部分失敗回復・構造体配置：1,154 assertions成功
- 警告0・エラー0のビルドを確認
- アイコン：16/24/32/48/64/128/256pxのICOを作成・目視確認
- Windows実行、Discord操作、IME動作は未検証

## Rust版 v0.2.0

- macOSでOS独立の6テスト成功。キー判定256組合せ、押下途中の遷移16,384組合せ、入力列、全ての部分注入境界、対象exe判定を含む。
- Windows x64向けの型チェック・Clippyによる静的検査を実施。
- [Windows CI（push）](https://github.com/spa77k/enter-guard/actions/runs/38589615067)と[PR CI](https://github.com/spa77k/enter-guard/actions/runs/38589664564)：整形、Clippy、Rust全7テスト、元.NET全1,154 assertions、配布ビルド、実行スモークテスト成功。
- Windows CIの実行確認はUI/フック/通知領域の初期化・初期停止・有効/停止切替・終了まで。Discord/IME操作を含まない。
- 配布exeを取得し、Windows x64 GUI・ICO/manifest埋め込み・動的CRT/.NET依存なし・SHA256一致を確認。`scripts/verify_exe.py`で再検証できる。
- 初回配布ビルド264,192 bytes（約258 KiB）。旧.NET exeは51,717,912 bytes。
- Discord実画面・日本語IME・DPI表示の人間による確認は未実施。

## 自動テスト

OSと独立したテストで、キー判定・修飾キーの組合せ・1押下1操作・通常/テンキーEnterの分離・自己注入識別・注入イベント順序・部分失敗時の後片付け・Windows x64 INPUT構造体の配置を検証します。

Windows向けのクロスビルド成功は、実機動作の確認ではありません。リリースノートには実行したテスト結果と未実施事項を分けて記録します。

## Windows実機チェックリスト（すべて未実施）

- [ ] Windows 11 x64で起動・日本語UI・独自アイコン表示
- [ ] 起動直後は停止中
- [ ] Discord通常版でEnter改行／Ctrl+Enter送信
- [ ] Microsoft IMEで変換中・候補選択中・確定後の操作
- [ ] Google日本語入力で変換中・候補選択中・確定後の操作
- [ ] Shift+EnterやCtrl+Shift+Enterなどを変更しない
- [ ] 他アプリへの影響なし
- [ ] 検索・モーダル・着信などチャット以外への影響確認
- [ ] 長押し・同時押し・前面切替・修飾キー切替
- [ ] 通常EnterとテンキーEnterの同時操作
- [ ] ロック／解除・スリープ復帰後のキー状態
- [ ] 注入失敗時の停止表示と修飾キー復元
- [ ] Ctrl+Alt+F10と他ソフトとの競合
- [ ] 通知領域から復帰・一時停止・終了
- [ ] 終了後にフックが解除される
- [ ] 拡大率125%/150%/200%で文字・ボタンが欠けない
- [ ] Discord再起動後の判定

人間の実機確認が終わるまでチェック済みにしません。

## v0.2.1 開発中の検証対象

- 設定parse/serialize、F8〜F11制約、unknown/version違い/破損/過大ファイル、保存と上書き。
- Windows smokeに停止操作の冪等性、異常状態で再有効化されないことを追加。
- 独立UIA fixtureは本文を読まず自分のEdit/password metadataを確認。チャットと検索が同じEdit型であることを検証。
- UIA/IMEの本体統合は未完成。fixture成功は実Discord/日本語IME検証を意味しない。詳しくはSAFETY-PROTOTYPE.md。
- 自動起動の実ユーザーOS登録、ホットキー競合の実操作、実Discord/IME操作、100〜200%表示確認は未実施。
- この追記時点では更新版WindowsCIの結果待ち。配布判定前に最終commitの実行結果を記録する。
