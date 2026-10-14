# 入力欄・IME判定の実験と公開条件

この実験は配布exeに組み込んでいません。通常チャット入力欄の自動判定、日本語IME変換中の除外は**未完成**です。既存のDiscord全体向けキー変換を、それらが完成したように置き換えません。

## 分かったこと

- `IUIAutomationTextEditPattern::GetActiveComposition` は対応プロバイダーがあるときだけ、現在の変換範囲を返します。TextEditPattern未対応・API失敗・不明を「変換していない」と解釈できません。範囲の有無だけを使い、本文を取得する `GetText` は呼ばない設計が必要です。
- DiscordはElectronです。「Edit型」「Discord.exe」だけではチャット・検索・ログインを区別できません。AutomationIdの固定値やNameの文字列を推測して許可することもしません。Nameは内容を含む可能性があるため取得しません。
- UIAは別MTAスレッドで実行すべきであり、低レベルキーフック内で同期問い合わせしません。ただし別スレッドの結果は古くなり得るので、時刻だけを信用して変換できません。フォーカス世代と入力直前の同一性確認が必要で、SendInputとの完全な原子性は保証されません。
- 不明時にEnterをそのまま通すと、Discord本来の送信が発生する可能性があります。「安全」「保護中」ではなく「変換対象を確認できない・保護なし」と表示すべきです。
- Windowsはフックを通知なしで解除することがあります。有効要求と動作確認を混同しません。再有効化時の再登録も、その後の健全性を保証しません。

## 実験テスト

`tests/SafetyPrototype` は独立したWindows WPF/UIAテストです。自分で作ったチャット用TextBox、検索用TextBox、PasswordBoxについて、本文を読まずに型・パスワード属性・プロセスを取得します。チャットと検索が同じEdit型である点、unknown IMEと古い世代を拒否する判定を検査します。自分のテスト欄には既知IDを付けますが、これをDiscord用IDの根拠にしません。

このテストはDiscord、実際のIME、候補ウィンドウ、変換確定キー、低レベルフックへの統合を検証しません。IME状態は明示的な模擬値です。

## 本体に公開する前に必要な確認

1. Windows 11 x64上の通常版Discordの実際のUIA提供内容を、本文/パスワードを取得せず確認する。
2. 通常チャットを識別するメタデータが、検索・ログイン・パスワード・編集ダイアログ・通話画面と区別できる根拠を得る。DM、サーバー、スレッド、言語・Discord更新差も確認する。
3. Microsoft日本語IMEで未変換入力・変換中・候補選択・確定・半角英数を検査する。TextEditPatternがないなら別の安全な根拠が必要。IMMだけでElectron/TSFの状態が取得できると仮定しない。
4. フォーカス変更直後、同じウィンドウ内の欄移動、UIA遅延/失敗、アプリ終了時に古い許可が使われないことを検査する。
5. 不明状態では変換しない旨と、通常Enter送信があり得る旨を画面に明示する。

## 一次資料

- https://learn.microsoft.com/en-us/windows/win32/api/uiautomationclient/nf-uiautomationclient-iuiautomationtexteditpattern-getactivecomposition
- https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-threading
- https://learn.microsoft.com/en-us/windows/win32/winmsg/lowlevelkeyboardproc
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput

確認日：2026-10-14。これらのAPI仕様はDiscordの実装対応を証明するものではありません。
