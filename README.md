# EnterGuard

![EnterGuard icon](assets/EnterGuard.png)

**Enterで改行、Ctrl+Enterで送信。** Windows向けの日本語キー変換ツールです。初版は通常版Discordのみ対応します。

**v0.1.0は試用版です。Windows実機・Discord・日本語IMEでの動作確認は未完了です。誤送信を完全に防ぐものではありません。**

## 動作

通常版の `Discord.exe` が最前面で、有効にしているときだけ動きます。

| 押すキー | 置き換え |
|---|---|
| Enter | Shift+Enter（通常のチャットでは改行） |
| Ctrl+Enter | 修飾キーなしEnter（通常のチャットでは送信） |
| Shift+Enter / Ctrl+Shift+Enter / AltやWindowsキー付きEnter | そのまま |

- 物理的な1回の押下につき1回だけ変換。長押しによる繰り返し送信を防ぎます。
- 日本語変換中のEnterにも作用します。IMEによっては確定や候補選択が普段と変わります。
- Ctrl+Enterが変換確定だけで終わっても、追加のEnterを自動送信しません。
- 入力欄を識別しないため、Discord内の検索・ダイアログ・通話画面にも作用します。Discord本来のCtrl+Enterによる通話応答などを上書きします。
- ブラウザー版Discord、PTB、Canary、スマートフォンには対応しません。
- 他アプリは対象外ですが、キー入力直前のフォーカス競合を完全には防げません。大切な会話では必ず動作確認してください。

## ダウンロード・使い方

1. [Releases](https://github.com/spa77k/enter-guard/releases)から `EnterGuard-win-x64.exe` を取得します。Windows 11 x64向けです。
2. exeを実行します。インストーラー・管理者権限・.NETの別インストールは不要です。
3. 起動直後は停止中です。注意事項を確認して、有効にするボタンを押します。
4. 自分だけのテスト用Discordチャンネルで、Enterの改行とCtrl+Enterの送信、日本語変換を確認します。
5. Ctrl+Alt+F10で有効／一時停止を切り替えます。他ソフトと競合する場合は、画面のボタンか通知領域メニューを使ってください。
6. 画面の終了ボタン、右上の×、通知領域メニューから終了できます。終了するとキー変換は解除されます。

コード署名はありません。Windowsが警告・ブロックする場合があります。警告を無視して実行することは勧めません。公開ソースの確認や、信頼できる環境でのビルドを検討してください。SHA256SUMS.txtで配布ファイルの一致を照合できますが、ハッシュ一致だけで安全性が保証されるわけではありません。

## 仕組みと制限

Windowsの低レベルキーボードフックでEnterを扱い、前面プロセス名がDiscordの場合だけ、本人のキー操作をSendInputで置き換えます。Ctrl+Enterでは押されている左右Ctrlを一時解放し、Enterの後に復元します。注入失敗時は自動停止し、Enterを再送せず警告します。

- Discordのファイル・プログラムは変更しません
- DiscordへのDLL注入、API操作、トークン取得は行いません
- メッセージ本文の取得、入力ログの記録、外部通信は行いません
- 自動起動登録や設定ファイルの保存は行いません
- アプリアイコンは独自作成で、Discordのロゴではありません

Windowsのフックには、処理時間超過時に通知なしで解除される制限があります。有効表示は現在のフック動作を保証しません。別のキー変換ソフトや人工的なキー入力、管理者権限で起動したDiscordとの併用も保証しません。入力が不安定になったら停止してください。

## Discordとの関係・規約

Discord非公式の独立したユーティリティです。クライアント改変やアカウントの自動操作を行わず、本人が押したキーだけを変換する設計です。ただし、外部キーツールを明示的に許可する公式資料は確認できていません。公式の承認・提携、規約上の許可やアカウントへの影響を保証するものではありません。資料確認日：2026年10月14日。

- [Discord利用規約](https://discord.com/terms)
- [クライアント改変に関する公式説明](https://discord.com/safety/platform-manipulation-policy-explainer-oct-2023)
- [Self-Botについて](https://support.discord.com/hc/en-us/articles/115002192352-Automated-User-Accounts-Self-Bots)
- [Discord Windowsホットキー](https://support.discord.com/hc/ja/articles/225977308--Windows-Discord%E3%81%AE%E3%83%9B%E3%83%83%E3%83%88%E3%82%AD%E3%83%BC)
- [Microsoft LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)
- [Microsoft SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

## ソースからビルド

.NET 10 SDKが必要です。

```powershell
dotnet run --project tests/GuardLogic.Tests -c Release
dotnet publish src/EnterGuard -c Release -r win-x64 -o artifacts/win-x64
Get-FileHash artifacts/win-x64/EnterGuard.exe -Algorithm SHA256
```

.NET 10ランタイムを同梱します。セキュリティ更新時は最新パッチのSDKで再ビルド・再配布する必要があります。単一exeは必要なネイティブファイルを一時領域に展開します。

[検証記録](docs/TESTING.md)を確認してください。MITライセンス。Discordの商標・権利はそれぞれの権利者に帰属します。
