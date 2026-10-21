# EnterGuard for Mac

Enterで改行、⌘ Enterで送信。通常版のデスクトップDiscordで使う、Rust製のキー変換アプリです。

## 入手方法

Mac版はソースコードを公開しています。アプリのダウンロード用リリースはまだありません。下の「ソースからビルド」の手順で、macOS上でアプリとZIPを作成できます。

- **Apple Silicon（M1以降）：** `aarch64-apple-darwin` → `EnterGuard-mac-arm64.zip`
- **Intel Mac：** `x86_64-apple-darwin` → `EnterGuard-mac-x64.zip`

対象はmacOS 13以降です。「このMacについて」の「チップ」または「プロセッサ」で種類を確認できます。

## 起動する

1. ZIPを展開し、`EnterGuard.app` を「アプリケーション」フォルダーへ移します。
2. `EnterGuard.app` を開きます。起動直後は停止状態です。
3. アプリの案内に従い、macOSの「システム設定」→「プライバシーとセキュリティ」→「アクセシビリティ」で、EnterGuardの操作を許可します。これはキー変換に必要な権限です。設定を変更するのは利用者本人です。
4. EnterGuardに戻って「有効にする」を押し、Discordの入力欄へ戻ります。

ビルドスクリプトはアプリにアドホック署名を付けます。AppleのDeveloper ID署名・公証は付いていないため、macOSが初回起動を止める場合があります。その場合は配布元を確認し、[Appleのアプリを安全に開く手順](https://support.apple.com/ja-jp/102445)に従ってください。Gatekeeperの無効化やターミナルでの保護解除は不要です。

## キー操作

| 操作 | Discordに届くキー |
|---|---|
| Enter | Shift+Enter（改行） |
| ⌘ Enter | Enter（送信） |
| Shift+Enter、ControlやOptionを含むEnter | そのまま |
| ⌘ Option F10 | EnterGuardを一時停止 |

長押しの繰り返しは抑え、1回の押下で1回だけ変換します。Macの設定によっては停止キーにFnも必要です。画面の「一時停止する」でも止められます。「終了する」または⌘ Qでアプリを終了します。赤い閉じるボタンは画面を隠すだけで、変換状態は変わりません。メニューの「画面を開く」で戻せます。停止キーは再開には使いません。

## どこで動くか

- EnterGuardを有効にし、通常版のDiscordが最前面にある間だけ変換します。他のアプリ、ブラウザー版Discord、PTB、Canaryは対象外です。
- **対象はDiscordのウィンドウ全体です。** チャット欄と検索欄・ダイアログは区別しません。検索や通話の操作前は一時停止してください。
- **日本語の変換確定・候補選択のEnterにも作用します。** IMEの変換中を自動で除外する機能はありません。普段どおりの変換確定・候補選択を行うときは一時停止してください。
- ⌘ Enterが変換確定だけで終わっても、追加のEnterは送信しません。
- キーを押す瞬間にアプリを切り替えると、判定と入力先がずれることがあります。アプリの切り替え中はEnterを押し続けないでください。
- 停止中・終了後はDiscord本来のEnterに戻ります。

## 停止・権限の問題

キー変換がmacOSによって停止された場合、EnterGuardは停止表示に切り替えます。勝手に変換を再開しません。Discordの入力状態を確認してから、EnterGuardを終了・再起動してから有効にしてください。権限を許可しても有効にできない場合は、一度EnterGuardを終了して起動し直します。

起動のたびに停止状態から始まります。自動起動の登録、セキュリティ設定の自動変更は行いません。

## プライバシー

入力本文を読み取ったり、キーの履歴を記録したり、外部へ送信したりしません。Discordのファイル変更・API操作・トークン取得も行いません。macOSのイベントタップで本人が押したEnterの修飾キーだけを書き換えます。

## ソースからビルド

macOS上でRust 1.99.0を使用します。

```sh
cd rust
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
MACOSX_DEPLOYMENT_TARGET=13.0 cargo build --release --locked --target aarch64-apple-darwin
cd ..
./scripts/package_macos.sh aarch64-apple-darwin
```

Intel版はターゲットを`x86_64-apple-darwin`に置き換えます。パッケージスクリプトは `artifacts/distribution/` にZIPとSHA256を作成します。Mac向けのGitHub Actions workflowは、この変更には含めていません。
