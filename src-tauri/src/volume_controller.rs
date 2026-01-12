//! Windows システムボリューム制御モジュール
//!
//! 別スレッドでCOMを初期化し、チャネル経由でボリューム操作を行います。
//! これにより、Tauriのメインスレッド（STA）との競合を回避します。

use once_cell::sync::Lazy;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

#[cfg(windows)]
use windows::{
    Win32::Media::Audio::{
        eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
    },
    Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
        COINIT_MULTITHREADED,
    },
};

/// ボリューム操作コマンド
enum VolumeCommand {
    GetVolume(Sender<Result<f32, String>>),
    SetVolume(f32, Sender<Result<f32, String>>),
    VolumeUp(Sender<Result<f32, String>>),
    VolumeDown(Sender<Result<f32, String>>),
    Shutdown,
}

/// グローバルなボリュームコントローラーへのチャネル
static VOLUME_CONTROLLER: Lazy<Mutex<Option<Sender<VolumeCommand>>>> =
    Lazy::new(|| Mutex::new(None));

/// ボリュームコントローラーを初期化
pub fn init_volume_controller() {
    let mut controller = VOLUME_CONTROLLER.lock().unwrap();
    if controller.is_some() {
        return; // 既に初期化済み
    }

    let (tx, rx) = mpsc::channel::<VolumeCommand>();
    *controller = Some(tx);

    // 別スレッドでCOMを初期化してボリューム操作を行う
    thread::spawn(move || {
        volume_worker_thread(rx);
    });
}

/// ボリュームワーカースレッド
#[cfg(windows)]
fn volume_worker_thread(rx: Receiver<VolumeCommand>) {
    unsafe {
        // MTAとしてCOMを初期化
        // CoInitializeExはHRESULTを返す。ok()でResult<(), Error>に変換
        if CoInitializeEx(None, COINIT_MULTITHREADED).is_err() {
            eprintln!("[VolumeController] COM initialization failed");
            return;
        }

        // オーディオエンドポイントを取得
        let endpoint = match get_audio_endpoint() {
            Ok(ep) => ep,
            Err(e) => {
                eprintln!("[VolumeController] Failed to get audio endpoint: {}", e);
                CoUninitialize();
                return;
            }
        };

        println!("[VolumeController] Volume controller initialized successfully");

        // メッセージループ
        while let Ok(cmd) = rx.recv() {
            match cmd {
                VolumeCommand::GetVolume(response_tx) => {
                    let result = endpoint
                        .GetMasterVolumeLevelScalar()
                        .map_err(|e| format!("Failed to get volume: {:?}", e));
                    let _ = response_tx.send(result);
                }
                VolumeCommand::SetVolume(level, response_tx) => {
                    let clamped = level.clamp(0.0, 1.0);
                    let result = endpoint
                        .SetMasterVolumeLevelScalar(clamped, std::ptr::null())
                        .map(|_| clamped)
                        .map_err(|e| format!("Failed to set volume: {:?}", e));
                    let _ = response_tx.send(result);
                }
                VolumeCommand::VolumeUp(response_tx) => {
                    let result = match endpoint.GetMasterVolumeLevelScalar() {
                        Ok(current) => {
                            let new_level = (current + 0.05).min(1.0);
                            match endpoint.SetMasterVolumeLevelScalar(new_level, std::ptr::null()) {
                                Ok(_) => Ok(new_level),
                                Err(e) => Err(format!("Failed to set volume: {:?}", e)),
                            }
                        }
                        Err(e) => Err(format!("Failed to get volume: {:?}", e)),
                    };
                    let _ = response_tx.send(result);
                }
                VolumeCommand::VolumeDown(response_tx) => {
                    let result = match endpoint.GetMasterVolumeLevelScalar() {
                        Ok(current) => {
                            let new_level = (current - 0.05).max(0.0);
                            match endpoint.SetMasterVolumeLevelScalar(new_level, std::ptr::null()) {
                                Ok(_) => Ok(new_level),
                                Err(e) => Err(format!("Failed to set volume: {:?}", e)),
                            }
                        }
                        Err(e) => Err(format!("Failed to get volume: {:?}", e)),
                    };
                    let _ = response_tx.send(result);
                }
                VolumeCommand::Shutdown => {
                    println!("[VolumeController] Shutting down");
                    break;
                }
            }
        }

        CoUninitialize();
    }
}

/// オーディオエンドポイントを取得
#[cfg(windows)]
unsafe fn get_audio_endpoint() -> Result<IAudioEndpointVolume, String> {
    // デバイス列挙子を作成
    let enumerator: IMMDeviceEnumerator =
        CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_INPROC_SERVER)
            .map_err(|e| format!("Failed to create device enumerator: {:?}", e))?;

    // デフォルト出力デバイスを取得
    let device = enumerator
        .GetDefaultAudioEndpoint(eRender, eConsole)
        .map_err(|e| format!("Failed to get default audio endpoint: {:?}", e))?;

    // IAudioEndpointVolumeを取得
    let endpoint: IAudioEndpointVolume = device
        .Activate(CLSCTX_INPROC_SERVER, None)
        .map_err(|e| format!("Failed to activate audio endpoint volume: {:?}", e))?;

    Ok(endpoint)
}

/// 非Windowsプラットフォーム用のスタブ
#[cfg(not(windows))]
fn volume_worker_thread(_rx: Receiver<VolumeCommand>) {
    eprintln!("[VolumeController] Volume control is only supported on Windows");
}

/// コマンドを送信して結果を待つヘルパー関数
fn send_command<F>(create_command: F) -> Result<f32, String>
where
    F: FnOnce(Sender<Result<f32, String>>) -> VolumeCommand,
{
    let controller = VOLUME_CONTROLLER.lock().unwrap();
    let tx = controller
        .as_ref()
        .ok_or("Volume controller not initialized")?;

    let (response_tx, response_rx) = mpsc::channel();
    tx.send(create_command(response_tx))
        .map_err(|_| "Failed to send command to volume controller")?;

    response_rx
        .recv()
        .map_err(|_| "Failed to receive response from volume controller")?
}

/// 現在のボリュームを取得 (0.0 - 1.0)
pub fn get_volume() -> Result<f32, String> {
    send_command(VolumeCommand::GetVolume)
}

/// ボリュームを設定 (0.0 - 1.0)
#[allow(dead_code)]
pub fn set_volume(level: f32) -> Result<f32, String> {
    send_command(|tx| VolumeCommand::SetVolume(level, tx))
}

/// ボリュームを5%上げる
pub fn volume_up() -> Result<f32, String> {
    send_command(VolumeCommand::VolumeUp)
}

/// ボリュームを5%下げる
pub fn volume_down() -> Result<f32, String> {
    send_command(VolumeCommand::VolumeDown)
}

/// ボリュームコントローラーをシャットダウン
#[allow(dead_code)]
pub fn shutdown_volume_controller() {
    if let Ok(controller) = VOLUME_CONTROLLER.lock() {
        if let Some(tx) = controller.as_ref() {
            let _ = tx.send(VolumeCommand::Shutdown);
        }
    }
}
