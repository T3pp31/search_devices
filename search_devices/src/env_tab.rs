//! 環境設定タブのUIを構築します。
//! 各種設定項目（繰り返し回数、実行間隔など）を配置します。

use fltk::{prelude::*, group::Group, input::IntInput, button::CheckButton, frame::Frame, enums::Align, widget_extends, menu::Choice, button::RadioRoundButton};

/// 環境設定タブを構築し、Groupを返します。
///
/// # 戻り値
/// * `Group` - 環境設定タブのグループ
pub fn build_env_tab() -> (Group, IntInput, IntInput, IntInput, IntInput, IntInput) {
    // タブ用グループ
    let grp = Group::new(0, 25, 500, 375, "設定"); // タブ名を「設定」に
    grp.begin();

    // 繰り返し回数
    let _frame_repeat = Frame::new(20, 30, 100, 25, "繰り返し回数:");
    let mut input_repeat = IntInput::new(130, 30, 60, 25, "");
    input_repeat.set_value("1");

    // 実行間隔
    let _frame_interval = Frame::new(20, 70, 100, 25, "実行間隔:");
    let mut input_interval = IntInput::new(130, 70, 60, 25, "");
    input_interval.set_value("1000");
    let _frame_interval_unit = Frame::new(200, 70, 40, 25, "ミリ秒");

    // ブロックサイズ
    let _frame_block = Frame::new(20, 110, 100, 25, "ブロックサイズ:");
    let mut input_block = IntInput::new(130, 110, 60, 25, "");
    input_block.set_value("64");
    let _frame_block_unit = Frame::new(200, 110, 40, 25, "バイト");

    // タイムアウト
    let _frame_timeout = Frame::new(20, 150, 100, 25, "タイムアウト:");
    let mut input_timeout = IntInput::new(130, 150, 60, 25, "");
    input_timeout.set_value("5000");
    let _frame_timeout_unit = Frame::new(200, 150, 40, 25, "ミリ秒");

    // TTL
    let _frame_ttl = Frame::new(20, 190, 100, 25, "TTL:");
    let mut input_ttl = IntInput::new(130, 190, 60, 25, "");
    input_ttl.set_value("255");

    grp.end();
    (
        grp,
        input_repeat,
        input_interval,
        input_block,
        input_timeout,
        input_ttl,
    )
}
