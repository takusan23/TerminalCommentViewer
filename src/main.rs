// JSONパースするやつ
use serde_json::json;

// スレッド。並列
use std::thread;

// CLIから実行するときの引数
use std::env;

// API叩きときに使う。HttpClient
use reqwest;

// こっからAPIのレスポンスの構造体。Rustくんはスネークケースで書けって言われてる（_で繋ぐやつ）からここで警告が出る
#[derive(serde::Serialize, serde::Deserialize)]
struct ProgramInfo {
    data: Data,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Data {
    rooms: Vec<Room>,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Room {
    webSocketUri: String,
    threadId: String,
    name: String,
}

// こっからコメントJSONの構造体
#[derive(serde::Serialize, serde::Deserialize)]
struct Comment {
    chat: Chat,
}
#[derive(serde::Serialize, serde::Deserialize)]
struct Chat {
    content: String,
    no: u32,              // 数値型の種類いっぱいでよくわからん
    premium: Option<u32>, // プレ垢で1。運コメで3？
}

/// # 実行方法
/// それぞれ置き換えて。
/// ```
/// cargo run {mail} {pass} {live_id}
/// ```
/// - `mail`     -ログインするのに使う
/// - `pass`     - ログインするのに使う
/// - `live_id`  - 生放送ID
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 起動時の引数。
    let args: Vec<String> = env::args().collect();
    let mail = &args[1];
    let pass = &args[2];
    let live_id = &args[3];
    // ログインしてユーザーセッションを取得する。なお毎回取得してるとログアウトされるので本当は保存されるべき。今回はしてない。
    let login_response = login(mail.to_string(), pass.to_string()).await;
    // ユーザーセッション
    let user_session = login_response.unwrap();
    // APIのURL。先頭はStringに変換する必要がある？
    let url = "https://live2.nicovideo.jp/watch/".to_string() + live_id + "/programinfo";
    // コメントサーバー情報取得APIを叩く。公式では使えない
    let client = reqwest::Client::new();
    let response = client
        .get(url.as_str())
        .header("Cookie", user_session)
        .send()
        .await?
        .text()
        .await?;
    // JSONパーサーへ。serde_json::Valueは使えなかった。
    let json: ProgramInfo = serde_json::from_str(&response).expect("Error");
    // コメントサーバーへ接続する。Vecにはfor_eachないの？
    for room in json.data.rooms {
        // コメントサーバーに接続すると無限ループへ入るので別スレッドを用意する。
        // なお move がないと roomへアクセスできない
        thread::spawn(move || {
            println!("{}", room.name);
            connect_room_server(&room.webSocketUri, &room.threadId, &room.name);
        });
    }
    // メイン関数を終わらせないための無限ループ。もっといい方法がありそう
    loop {}
    // asyncの返り値？
    Ok(())
}

/// ニコニコにログインする関数。リダイレクト禁止にしてステータスコードを302にしないと取れない。
///
/// この関数を使うとたまによく他デバイス（PC、スマホ等）がログアウトされる（同時ログインが少ない）
///
/// なので本来はユーザーセッションが生きてる間は保存しておくべき。でも今回はしていない。
///
/// # 引数
/// * `mail` - ログインするときに使うメアド
/// * `pass` - ログインで使うパスワード
///
/// # 返り値
/// * ユーザーセッション。`unwrap()`でResult型から取り出すことができます。（詳しくは知らん）
///
async fn login(mail: String, pass: String) -> Result<String, Box<dyn std::error::Error>> {
    // リダイレクト禁止
    let custom = reqwest::redirect::Policy::custom(|attempt| attempt.stop());
    let client = reqwest::Client::builder().redirect(custom).build()?;
    // POSTする中身
    let params = [("mail_tel", mail), ("password", pass)];
    // リクエスト
    let respone = client
        .post("https://secure.nicovideo.jp/secure/login?site=niconico")
        .form(&params)
        .header("User-Agent", "TerminalCommentViewer;@takusan_23")
        .send()
        .await?;
    // ユーザーセッション取り出し。今回はある前提で
    let mut user_session = String::new();
    for (key, value) in respone.headers() {
        // user_sessionを取り出す
        if key == "set-cookie"
            && value.to_str().unwrap().contains("user_session")
            && !value.to_str().unwrap().contains("user_session_secure")
        {
            // ; で分割して取り出す
            let cookie = value.to_str().unwrap();
            let cookie_list: Vec<&str> = cookie.split(';').collect();
            user_session = cookie_list[0].to_string();
        }
    }
    // 返り値
    Ok(user_session.to_string())
}

/// コメントサーバーへ接続する関数
/// # 引数
/// * `address`   - WebSocketアドレス
/// * `thread`    - API叩いたときに取れる
/// * `room_name` - 部屋名。まあなんでも良い。
fn connect_room_server(address: &String, thread: &String, room_name: &String) {
    // 送るJSON
    let send_data = json!({
        "thread":{
            "thread":thread,
            "version":"20061206",
            "service":"LIVE",
            "score":0,
            "res_from":0
        }
    });
    // リクエスト。msg.nicovideo.jp#jsonを指定するためにhttp::request::Requestを使っている。
    let request = http::request::Request::get(address)
        .header(
            "Sec-WebSocket-Protocol",
            "msg.nicovideo.jp#json".to_string(),
        )
        .body(())
        .unwrap();
    // WebSocket接続
    let (mut socket, _response) = tungstenite::connect(request).expect("Can't connect");
    // 一度だけ送信
    let send_result = socket.write_message(tungstenite::Message::Text(send_data.to_string()));
    println!("JSON Send {}", send_result.is_ok());
    loop {
        // メッセージを受け取る。無限ループ
        let msg = socket.read_message().expect("Error reading message");
        let json_string = msg.to_text().unwrap();
        // threadとか流れてくるので対策
        if json_string.contains("chat") {
            // JSONパース
            let chat_json: Comment = serde_json::from_str(json_string).expect("Error");
            let comment_no = chat_json.chat.no;
            let content = chat_json.chat.content;
            // Option型を試すためにわざわざプレ垢かどうかを取る。unwrap_orを使えば失敗時は引数の値を使ってくれる。
            let is_premium = chat_json.chat.premium.unwrap_or(0) == 1;
            let premium_string = if is_premium { "P" } else { "" };
            // 出力
            println!(
                "{}\t{}\t{}\t{}",
                room_name, comment_no, premium_string, content
            );
        }
    }
}
