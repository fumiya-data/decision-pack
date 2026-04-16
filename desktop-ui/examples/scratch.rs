// このサンプルで必要になるウィジェットのコンストラクタを取り込みます。
// 使用するウィジェットは次のとおりです。
// - `button`: クリック可能なボタンを作る
// - `column`: ウィジェットを縦方向に配置する
// - `mouse_area`: ウィジェット領域の周囲でマウス入力を捕捉する
// - `pick_list`: 選択用のドロップダウン一覧を表示する
// - `text`: 画面上に文字列を表示する
use iced::widget::{button, column, mouse_area, pick_list, text};

// Iced の基本型を取り込みます。
// - `Element`: `view` が返す汎用 UI ツリー
// - `mouse`: スクロール量などのマウス関連イベント型
// - `Task`: `update` から開始される副作用や非同期処理
use iced::{Element, Task, Theme, mouse};

// `Default` を derive して `App::default()` から初期状態を生成できるようにします。
#[derive(Default)]
struct App {
    // 表示内容を決めるカウンター状態です。
    value: i32,

    // 現在選択されているテーマです。
    theme: Theme,

    // テーマ用ホイール入力の疑似フォーカス状態です。
    theme_wheel_active: bool,

    // pick list メニューが開いているかどうかを保持します。
    theme_menu_open: bool,
}

// `Message` は UI が発行できるイベントです。
#[derive(Debug, Clone)]
enum Message {
    // 加算ボタン押下時。
    Inc,

    // 減算ボタン押下時。
    Dec,

    // ドロップダウンからテーマを選択したとき。
    ThemeSelected(Theme),

    // テーマセレクタ領域がホイール入力用にアクティブ化されたとき。
    ThemeWheelActivated,

    // テーマセレクタ上でホイールが使われたとき。
    ThemeWheelScrolled(mouse::ScrollDelta),

    // pick list メニューが開いたとき。
    ThemeMenuOpened,

    // pick list メニューが閉じたとき。
    ThemeMenuClosed,
}

// `update` はイベントを処理し、アプリケーション状態を書き換えます。
fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        // 加算ボタンが押されたら、状態に 1 を足します。
        Message::Inc => {
            app.value += 1;
            app.theme_wheel_active = false;
        }

        // 減算ボタンが押されたら、状態から 1 を引きます。
        Message::Dec => {
            app.value -= 1;
            app.theme_wheel_active = false;
        }

        // テーマ選択時は現在のテーマを差し替えます。
        Message::ThemeSelected(theme) => {
            app.theme = theme;
        }

        // セレクタ領域でホイール切り替えを有効化します。
        Message::ThemeWheelActivated => {
            app.theme_wheel_active = true;
        }

        // セレクタがアクティブなら、ホイール方向に応じてテーマを巡回させます。
        Message::ThemeWheelScrolled(delta) => {
            if app.theme_wheel_active && !app.theme_menu_open {
                if let Some(step) = theme_scroll_step(delta) {
                    app.theme = cycle_theme(&app.theme, step);
                }
            }
        }

        // pick list メニューが開いている状態を記録します。
        Message::ThemeMenuOpened => {
            app.theme_wheel_active = true;
            app.theme_menu_open = true;
        }

        // pick list メニューが閉じた状態を記録します。
        Message::ThemeMenuClosed => {
            app.theme_menu_open = false;
        }
    }

    // このメッセージ処理のあとに必要な非同期処理はありません。
    Task::none()
}

// `view` は現在のアプリケーション状態に対応する UI を記述します。
fn view(app: &App) -> Element<'_, Message> {
    let theme_status = if app.theme_wheel_active {
        "テーマ用ホイール入力: 有効"
    } else {
        "テーマ用ホイール入力: 無効"
    };

    let theme_hint = if app.theme_menu_open {
        "テーマメニューを開いています。"
    } else {
        "この領域をクリックしたあと、マウスホイールでテーマを切り替えます。"
    };

    let theme_selector = mouse_area(
        column![
            // 現在のテーマ名を表示します。
            text(format!("現在のテーマ: {}", app.theme)),
            // ホイール切り替えが有効かどうかを表示します。
            text(theme_status),
            // セレクタ領域の使い方を短く表示します。
            text(theme_hint),
            // `Theme::ALL` からドロップダウン一覧を作成します。
            pick_list(Theme::ALL, Some(app.theme.clone()), Message::ThemeSelected)
                .on_open(Message::ThemeMenuOpened)
                .on_close(Message::ThemeMenuClosed),
        ]
        .spacing(6),
    )
    // この領域をクリックするとホイールによるテーマ切り替えが有効になります。
    .on_press(Message::ThemeWheelActivated)
    // この領域上でスクロールするとホイール量が `update` に送られます。
    .on_scroll(Message::ThemeWheelScrolled);

    column![
        // `app` に保存されている現在のカウンター値を表示します。
        text("練習用カウンター"),
        text(format!("値: {}", app.value)),
        theme_selector,
        // クリックされると `Message::Inc` が `update` に送られます。
        button("1 増やす").on_press(Message::Inc),
        // 同様に、このボタンは `Message::Dec` を送ります。
        button("1 減らす").on_press(Message::Dec),
    ]
    // `column` 内の各ウィジェットの間に縦方向の余白を追加します。
    .spacing(10)
    // 列全体の外周に余白を追加します。
    .padding(20)
    // `column` ウィジェットを汎用戻り値 `Element` に変換します。
    .into()
}

// スクロール量をテーマ巡回用の単一ステップへ変換します。
// Y 方向の負値は前進、正値は後退として扱います。
fn theme_scroll_step(delta: mouse::ScrollDelta) -> Option<i32> {
    let y = match delta {
        mouse::ScrollDelta::Lines { y, .. } => y,
        mouse::ScrollDelta::Pixels { y, .. } => y,
    };

    if y < 0.0 {
        Some(1)
    } else if y > 0.0 {
        Some(-1)
    } else {
        None
    }
}

// `Theme::ALL` の中で `step` 個だけ移動した次の組み込みテーマを返します。
// リストの両端では循環するように折り返します。
fn cycle_theme(current: &Theme, step: i32) -> Theme {
    let themes = Theme::ALL;

    let Some(index) = themes.iter().position(|theme| theme == current) else {
        return Theme::default();
    };

    let len = themes.len() as i32;
    let next = (index as i32 + step).rem_euclid(len) as usize;

    themes[next].clone()
}

// `main` はプログラムの開始地点です。
fn main() -> iced::Result {
    // `iced::application(title, update, view)` で中核要素を結び付けます。
    iced::application("練習用カウンター", update, view)
        // 現在の状態から有効なテーマを読み取ります。
        .theme(|app| app.theme.clone())
        // `run_with` で初期状態と起動時タスクを与えて開始します。
        .run_with(|| (App::default(), iced::Task::none()))
}
