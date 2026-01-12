import { listen } from "@tauri-apps/api/event";
import { getVolume, volumeUp, volumeDown, volumeToPercent } from "./volume-service";

// DOM要素
let volumeDisplay: HTMLElement | null = null;
let volumeBar: HTMLElement | null = null;
let statusMessage: HTMLElement | null = null;
let volumeUpBtn: HTMLElement | null = null;
let volumeDownBtn: HTMLElement | null = null;

/**
 * ボリューム表示を更新
 */
async function updateVolumeDisplay(): Promise<void> {
  const result = await getVolume();

  result.match(
    (volume) => {
      const percent = volumeToPercent(volume);
      if (volumeDisplay) {
        volumeDisplay.textContent = `${percent}%`;
      }
      if (volumeBar) {
        volumeBar.style.width = `${percent}%`;
      }
      if (statusMessage) {
        statusMessage.textContent = "";
        statusMessage.className = "status-message";
      }
    },
    (error) => {
      if (statusMessage) {
        statusMessage.textContent = `エラー: ${error.message}`;
        statusMessage.className = "status-message error";
      }
    }
  );
}

/**
 * ボリュームアップ処理
 */
async function handleVolumeUp(): Promise<void> {
  const result = await volumeUp();

  result.match(
    (volume) => {
      const percent = volumeToPercent(volume);
      if (volumeDisplay) {
        volumeDisplay.textContent = `${percent}%`;
      }
      if (volumeBar) {
        volumeBar.style.width = `${percent}%`;
      }
      showFeedback("up");
    },
    (error) => {
      if (statusMessage) {
        statusMessage.textContent = `エラー: ${error.message}`;
        statusMessage.className = "status-message error";
      }
    }
  );
}

/**
 * ボリュームダウン処理
 */
async function handleVolumeDown(): Promise<void> {
  const result = await volumeDown();

  result.match(
    (volume) => {
      const percent = volumeToPercent(volume);
      if (volumeDisplay) {
        volumeDisplay.textContent = `${percent}%`;
      }
      if (volumeBar) {
        volumeBar.style.width = `${percent}%`;
      }
      showFeedback("down");
    },
    (error) => {
      if (statusMessage) {
        statusMessage.textContent = `エラー: ${error.message}`;
        statusMessage.className = "status-message error";
      }
    }
  );
}

/**
 * ボリューム変更フィードバックを表示
 */
function showFeedback(direction: "up" | "down"): void {
  const feedback = document.querySelector(".feedback-indicator");
  if (feedback) {
    feedback.textContent = direction === "up" ? "🔊 +" : "🔉 -";
    feedback.classList.add("active");
    setTimeout(() => {
      feedback.classList.remove("active");
    }, 300);
  }
}

/**
 * アプリケーション初期化
 */
async function init(): Promise<void> {
  // DOM要素を取得
  volumeDisplay = document.getElementById("volume-display");
  volumeBar = document.getElementById("volume-bar");
  statusMessage = document.getElementById("status-message");
  volumeUpBtn = document.getElementById("volume-up-btn");
  volumeDownBtn = document.getElementById("volume-down-btn");

  // 初期ボリュームを表示
  await updateVolumeDisplay();

  // バックエンドからのボリューム変更イベントをリッスン
  await listen("volume-changed", async () => {
    await updateVolumeDisplay();
  });

  // ボタンクリックイベント
  volumeUpBtn?.addEventListener("click", handleVolumeUp);
  volumeDownBtn?.addEventListener("click", handleVolumeDown);
}

// DOM読み込み完了後に初期化
window.addEventListener("DOMContentLoaded", init);
