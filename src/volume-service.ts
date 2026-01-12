import { invoke } from "@tauri-apps/api/core";
import { Result, ResultAsync } from "neverthrow";

export type VolumeError = {
    type: "invoke_error";
    message: string;
};

/**
 * ボリュームをアップする
 */
export function volumeUp(): ResultAsync<number, VolumeError> {
    return ResultAsync.fromPromise(
        invoke<number>("volume_up"),
        (error): VolumeError => ({
            type: "invoke_error",
            message: String(error),
        })
    );
}

/**
 * ボリュームをダウンする
 */
export function volumeDown(): ResultAsync<number, VolumeError> {
    return ResultAsync.fromPromise(
        invoke<number>("volume_down"),
        (error): VolumeError => ({
            type: "invoke_error",
            message: String(error),
        })
    );
}

/**
 * 現在のボリュームを取得
 */
export function getVolume(): ResultAsync<number, VolumeError> {
    return ResultAsync.fromPromise(
        invoke<number>("get_volume"),
        (error): VolumeError => ({
            type: "invoke_error",
            message: String(error),
        })
    );
}

/**
 * ボリューム値をパーセントに変換
 */
export function volumeToPercent(volume: number): number {
    return Math.round(volume * 100);
}

/**
 * ボリューム取得結果を処理
 */
export function handleVolumeResult(
    result: Result<number, VolumeError>
): { success: true; volume: number } | { success: false; error: string } {
    return result.match(
        (volume) => ({ success: true as const, volume }),
        (error) => ({ success: false as const, error: error.message })
    );
}
