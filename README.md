# EnterGuard

![EnterGuard icon](assets/EnterGuard.png)

**Enterで改行、Ctrl+Enterで送信。** Windows向けの日本語キー変換ツールです。初版は通常版Discordのみ対応します。

**v0.2.1（Rust版）は試用版です。Windows実機・Discord・日本語IMEでの動作確認は未完了です。誤送信を完全に防ぐものではありません。**

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
5. Ctrl+Alt+F10は停止専用です。再開は画面か通知領域メニューから行います。「設定」でCtrl+Alt+F8〜F12から停止キーを選べます。競合時は以前のキーを維持し、登録できない場合は画面に表示します。
6. 画面の終了ボタン、右上の×、通知領域メニューから終了できます。終了するとキー変換は解除されます。

コード署名なし。Windowsの警告が出る場合があります。

Rust版でもキー変換仕様はv0.1.0と同じです。Discord内の入力欄・パスワード欄を識別しません。ログインやパスワード入力の前には必ず停止してください。

## 仕組みと制限

Windowsの低レベルキーボードフックでEnterを扱い、前面プロセス名がDiscordの場合だけ、本人のキー操作をSendInputで置き換えます。Ctrl+Enterでは押されている左右Ctrlを一時解放し、Enterの後に復元します。注入失敗時は自動停止し、Enterを再送せず警告します。異常後は画面や停止キーから再有効化できません。アプリを終了・再起動して状況を確認してください。再有効化時はフックを登録し直しますが、その後の動作を保証するものではありません。

- Discordのファイル・プログラムは変更しません
- DiscordへのDLL注入、API操作、トークン取得は行いません
- メッセージ本文の取得、入力ログの記録、外部通信は行いません
- 停止キーだけを `%LOCALAPPDATA%\EnterGuard\settings.txt` に保存します。有効状態は保存せず、常に停止状態で起動します
- 「設定」で選んだ場合だけ、本人のWindowsログイン時に起動する設定をHKCUのRunへ登録します。管理者権限不要。チェックを外すと解除します。exeを移動する前に解除してください。既存の別パスの登録は勝手に上書きしません
- アプリアイコンは独自作成で、Discordのロゴではありません

Windowsのフックには、処理時間超過時に通知なしで解除される制限があります。有効表示は現在のフック動作を保証しません。別のキー変換ソフトや人工的なキー入力、管理者権限で起動したDiscordとの併用も保証しません。入力が不安定になったら停止してください。

## 今回の改善と未完成の項目

v0.2.1では設定保存・停止専用キーの変更・任意自動起動・異常後の再開制限を追加しました。画面の「変換を要求中」はフックが動作し続けていることや誤送信防止を保証する表示ではありません。

**通常チャット入力欄だけの判定とIME変換中の自動除外は未完成です。** 配布exeは従来どおりDiscord全体が対象です。独立した[UIA判定実験と公開条件](docs/SAFETY-PROTOTYPE.md)を用意していますが、実Discordの検証が終わるまでは本体に組み込みません。検索・ログイン・IME変換時には停止してください。

## Discordとの関係・規約

Discord非公式の独立したユーティリティです。クライアント改変やアカウントの自動操作を行わず、本人が押したキーだけを変換する設計です。ただし、外部キーツールを明示的に許可する公式資料は確認できていません。公式の承認・提携、規約上の許可やアカウントへの影響を保証するものではありません。資料確認日：2026年10月14日。

- [Discord利用規約](https://discord.com/terms)
- [クライアント改変に関する公式説明](https://discord.com/safety/platform-manipulation-policy-explainer-oct-2023)
- [Self-Botについて](https://support.discord.com/hc/en-us/articles/115002192352-Automated-User-Accounts-Self-Bots)
- [Discord Windowsホットキー](https://support.discord.com/hc/ja/articles/225977308--Windows-Discord%E3%81%AE%E3%83%9B%E3%83%83%E3%83%88%E3%82%AD%E3%83%BC)
- [Microsoft LowLevelKeyboardProc](https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc)
- [Microsoft SendInput](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput)

## ソースからビルド

Rust版は `rust/` にあります。Windows 11 x64とVisual Studio Build ToolsのC++/Windows SDKが必要です。Rust 1.99.0を固定し、依存はCargo.lockで固定しています。

```powershell
cd rust
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
$env:RUSTFLAGS = "-C target-feature=+crt-static"
cargo build --release --locked --target x86_64-pc-windows-msvc
$process = Start-Process target/x86_64-pc-windows-msvc/release/enter-guard.exe -ArgumentList '--smoke-test' -PassThru -Wait
$process.ExitCode # 0が成功。Discord/IMEを操作するテストではありません。
```

Windows APIによる日本語UI・通知領域・フックと独自アイコンを単一exeに含めます。.NET不要、CRTは静的リンクします。ビルド依存の更新時は再検証・再配布してください。macOS上ではOS独立のテストとWindows向け型チェックが可能ですが、exeの配布ビルドはWindows上で行います。

GitHub ActionsではRustテスト・静的検査・既存.NETテスト・Windows x64の配布ビルド・起動/停止/終了の自動スモークテストを行い、exeとSHA256を成果物に保存します。

### 旧.NET版（v0.1.0）

既存の `src/EnterGuard` と `tests/GuardLogic.Tests`、v0.1.0リリースを保持しています。復旧時は旧リリースから取得できます。旧版のビルドには.NET 10 SDKが必要です。

```powershell
dotnet run --project tests/GuardLogic.Tests -c Release
dotnet publish src/EnterGuard -c Release -r win-x64 -o artifacts/win-x64
Get-FileHash artifacts/win-x64/EnterGuard.exe -Algorithm SHA256
```

.NET 10ランタイムを同梱します。セキュリティ更新時は最新パッチのSDKで再ビルド・再配布する必要があります。単一exeは必要なネイティブファイルを一時領域に展開します。

[検証記録](docs/TESTING.md)を確認してください。MITライセンス。Discordの商標・権利はそれぞれの権利者に帰属します。
