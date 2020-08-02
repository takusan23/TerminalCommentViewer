# TerminalCommentViewer
Rust製ニコ生のコメビュ。   
Rustを書いてみたかった。   
Rustくんで作った(CLI)アプリは配布が簡単って聞いたんですけど**Windowsだとそんなことなかったです。**  

Linuxとかだと話が変わってくるんだと思いました。

# 使い方

## なんかしらんけど VCRUNTIME140.dll が必要な模様
**「配布が簡単」とはこれいかに**  
てなわけで https://visualstudio.microsoft.com/ja/downloads/ から `Visual Studio 2019 の Microsoft Visual C++ 再頒布可能パッケージ` を入れる必要がある模様（要検証）

これ使うと毎回ログインするので使わないほうが良い。  
どうしても使いたい場合は

```
cargo run メアド パスワード 番組ID
```

で使えます。（使わなくていい）

`terminal_comment_viewer.exe`のときは

```
terminal_comment_viewer.exe メアド パスワード 番組ID
```

です。（使わなくていい）

# りりーす
```
cargo build --release
```