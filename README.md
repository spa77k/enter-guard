# EnterGuard

![EnterGuard icon](assets/EnterGuard.png)

**Enterで改行、Ctrl+Enterで送信。** Windows向けの日本語キー変換ツールです。初版は通常版Discordのみ対応します。

**v0.2.1（Rust版）は試用版です。誤送信を完全に防ぐものではありません。**

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

### 1. Windows用のexeをダウンロードする

1. [v0.2.1のダウンロードページ](https://github.com/spa77k/enter-guard/releases/tag/v0.2.1)を開きます。
2. ページ下部の **Assets** から **`EnterGuard-win-x64.exe`** をクリックします。`Source code` は開発者向けなので、使うだけなら不要です。
3. ダウンロードしたexeを、保管しておきたいフォルダーに置きます。

対象は **Windows 11 x64＋通常版のデスクトップDiscord** です。macOS、スマートフォン、ブラウザー版Discord、Discord PTB／Canaryには対応していません。インストーラー・管理者権限・.NETの別インストールは不要です。

> コード署名のない試用版なので、Windowsの警告が出る場合があります。配布元とファイル名を確認し、不安がある場合は実行を中止してください。

### 2. 起動して、Discordで使える状態にする

1. `EnterGuard-win-x64.exe` をダブルクリックします。
2. 画面に **「停止中：Enterの動作を変更していません」** と表示されます。この時点ではDiscordのキー操作は変わりません。
3. 画面の注意事項を読み、**「注意事項を確認して有効にする」** を押します。
4. 表示が **「変換を要求中（動作保証なし）」** に変わったら、Discordのチャット画面に戻り、メッセージ入力欄をクリックします。

**EnterGuardを有効にしたうえで、Discordを最前面にしている間だけ変換します。** 「変換を要求中」は設定上の状態であり、変換が確実に動作していることを保証する表示ではありません。

### 3. 自分だけのチャンネルで試す

最初は他の人に届かないテスト用チャンネルで、次の順に確認してください。まずは日本語の変換操作を含まない半角英数字で試します。

1. `test` と入力して **Enter** を押す → 送信されず、入力欄の中で改行されることを確認します。
2. 続けて `123` と入力する → 2行の文章になっていることを確認します。
3. **Ctrlを押しながらEnter** を押す → 2行の文章が1つのメッセージとして送信されることを確認します。

通常のチャットでは、**書いている途中の改行はEnter、書き終わって送信するときはCtrl+Enter** です。期待どおりに動かない場合は一時停止し、大切な会話では使わないでください。

#### 日本語入力・検索・ログイン時の注意

**日本語IMEの変換中を自動判定して除外する機能は、まだありません。** 変換確定や候補選択のEnterにも作用するため、変換確定が必要なときは一時停止してください。停止中のEnterはDiscord本来の動作に戻り、確定後にもう一度押すと送信される場合があります。

Ctrl+Enterを押しても日本語の変換確定だけで終わった場合、EnterGuardは追加のEnterを自動で送りません。連打せず、入力欄と送信済みメッセージを確認してください。

また、チャット欄と検索欄・ログイン欄などの区別もできません。**検索、ログイン、パスワード入力、ダイアログ操作、通話応答の前にも停止してください。** Discord本来のCtrl+Enterによる通話応答などを上書きします。

### 4. 一時停止する・再開する・終了する

| やりたいこと | 操作 |
|---|---|
| すぐに一時停止 | Ctrl+Alt+F10を同時に押す（初期設定） |
| 画面から一時停止 | 「一時停止する」を押す |
| 再開 | 画面の「注意事項を確認して有効にする」を押す |
| 画面だけ隠して使い続ける | 「通知領域にしまう」を押す。有効／停止の状態は変わりません |
| 完全に終了 | 「終了する」、右上の×、または通知領域メニューの「終了」 |

**停止キーは停止専用です。もう一度押しても再開しません。** 一時停止・終了するとキー変換が解除され、Discord本来のEnter操作に戻ります。

通知領域はWindowsのタスクバーの時計付近です。アイコンが見つからない場合は、隠れているアイコンを表示する **「^」** を確認してください。EnterGuardのアイコンを右クリックすると **「画面を開く」「有効／一時停止」「設定（変更時は停止）」「終了」** を選べます。アイコンのダブルクリックでも画面を開けます。

### 5. 停止キー・Windows起動時の設定を変える

画面の **「設定」** を押すとメニューが開きます。**設定を開いた時点で一時停止します。** 変更しなかった場合も、使い続けるには自分で再開してください。

- **停止キー：** `Ctrl+Alt+F8`、`Ctrl+Alt+F9`、`Ctrl+Alt+F10`、`Ctrl+Alt+F11` から選べます。選んだキーは次回起動時にも使われます。他のソフトと競合して登録できない場合は警告を表示し、以前のキーを維持します。
- **自動起動：** **「Windowsログイン時に起動（必ず停止状態）」** を選ぶと、次回のWindowsログイン時に起動します。チェックを外すと解除します。**自動起動しても変換は有効になりません。** 毎回、注意事項を確認して有効にしてください。
- **exeを移動・削除する前：** 自動起動を設定していた場合は、先にチェックを外して解除してください。

### 困ったとき

- **Enterで送信されてしまう：** 停止中になっていないか、通常版のデスクトップDiscordを最前面にしているかを確認してください。有効表示でも動作が保証されるわけではありません。大切な文章で再試行せず、テスト用チャンネルで確認してください。
- **停止キーが効かない／登録できないと表示される：** 画面の「一時停止する」か通知領域のメニューで停止してください。必要なら「設定」で別の停止キーを選びます。
- **「すでに起動しています」と出る：** 通知領域のEnterGuardアイコンから「画面を開く」を選びます。
- **異常や入力の置き換え失敗が表示される：** 入力を止め、Discordで送信済みかどうかを先に確認してください。異常後はそのまま再開できません。EnterGuardを終了して起動し直し、テスト用チャンネルで確認してください。

Rust版でもキー変換仕様はv0.1.0と同じです。

## 仕組みと制限

Windowsの低レベルキーボードフックでEnterを扱い、前面プロセス名がDiscordの場合だけ、本人のキー操作をSendInputで置き換えます。Ctrl+Enterでは押されている左右Ctrlを一時解放し、Enterの後に復元します。注入失敗時は自動停止し、Enterを再送せず警告します。異常後は画面や停止キーから再有効化できません。アプリを終了・再起動して状況を確認してください。再有効化時はフックを登録し直しますが、その後の動作を保証するものではありません。

- Discordのファイル・プログラムは変更しません
- DiscordへのDLL注入、API操作、トークン取得は行いません
- メッセージ本文の取得、入力ログの記録、外部通信は行いません
- 停止キーだけを `%LOCALAPPDATA%\EnterGuard\settings.txt` に保存します。有効状態は保存せず、常に停止状態で起動します
- 「設定」で選んだ場合だけ、本人のWindowsログイン時に起動する設定をHKCUのRunへ登録します。管理者権限不要。チェックを外すと解除します。exeを移動する前に解除してください。既存の別パスの登録は勝手に上書きしません
- アプリアイコンは独自作成で、Discordのロゴではありません

Windowsのフックには、処理時間超過時に通知なしで解除される制限があります。有効表示は現在のフック動作を保証しません。別のキー変換ソフトや人工的なキー入力、管理者権限で起動したDiscordとの併用も保証しません。入力が不安定になったら停止してください。

## 今回の改善と制限

v0.2.1では設定保存・停止専用キーの変更・任意自動起動・異常後の再開制限を追加しました。画面の「変換を要求中」はフックが動作し続けていることや誤送信防止を保証する表示ではありません。

**通常チャット入力欄だけを判定する機能と、IME変換中を自動で除外する機能はありません。** 配布exeは従来どおりDiscord全体が対象です。検索・ログイン・IME変換時には停止してください。

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
