mod volume_controller;

use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

/// ボリュームを増加させる
#[tauri::command]
fn volume_up() -> Result<f32, String> {
    volume_controller::volume_up()
}

/// ボリュームを減少させる
#[tauri::command]
fn volume_down() -> Result<f32, String> {
    volume_controller::volume_down()
}

/// 現在のボリュームを取得
#[tauri::command]
fn get_volume() -> Result<f32, String> {
    volume_controller::get_volume()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ショートカットを定義
    let shortcut_up = Shortcut::new(Some(Modifiers::ALT), Code::ArrowUp);
    let shortcut_down = Shortcut::new(Some(Modifiers::ALT), Code::ArrowDown);

    let mut builder = tauri::Builder::default();

    // MCP Bridge プラグイン (デバッグビルドのみ)
    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if shortcut == &shortcut_up {
                            // Alt+↑: ボリュームアップ
                            let _ = volume_controller::volume_up();
                            let _ = app.emit("volume-changed", ());
                        } else if shortcut == &shortcut_down {
                            // Alt+↓: ボリュームダウン
                            let _ = volume_controller::volume_down();
                            let _ = app.emit("volume-changed", ());
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            // ボリュームコントローラーを初期化
            volume_controller::init_volume_controller();

            // グローバルショートカットを登録（失敗しても続行）
            if let Err(e) = app.global_shortcut().register(shortcut_up) {
                eprintln!("Failed to register shortcut_up: {:?}", e);
            }
            if let Err(e) = app.global_shortcut().register(shortcut_down) {
                eprintln!("Failed to register shortcut_down: {:?}", e);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![volume_up, volume_down, get_volume])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
