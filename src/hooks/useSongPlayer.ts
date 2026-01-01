import { useState, useRef, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { SongData, PlaybackPayload } from "@/types";

export function useSongPlayer(onSongEnd: () => void) {
    const [isPlaying, setIsPlaying] = useState(false);
    const [progress, setProgress] = useState(0);
    const playingSongIdRef = useRef<number | null>(null);

    const playSong = useCallback(
        async (song: SongData) => {
            try {
                setIsPlaying(true);
                setProgress(0);
                playingSongIdRef.current = song.id;

                const onEvent = new Channel<PlaybackPayload>();
                onEvent.onmessage = (payload) => {
                    if (playingSongIdRef.current !== song.id) return;

                    const { progress_frames, total_frames, is_finished } =
                        payload;
                    if (is_finished) {
                        onSongEnd();
                    } else if (total_frames > 0) {
                        setProgress(progress_frames / total_frames);
                    }
                };

                await invoke("song_start", {
                    id: song.id,
                    onEvent,
                });
            } catch (e) {
                console.error(e);
                toast.error("Failed to play song", {
                    description: String(e),
                });
                setIsPlaying(false);
            }
        },
        [onSongEnd],
    );

    const toggle = useCallback(async (shouldPlay: boolean) => {
        try {
            if (shouldPlay) {
                await invoke("song_play");
                setIsPlaying(true);
            } else {
                await invoke("song_pause");
                setIsPlaying(false);
            }
        } catch (e) {
            console.error(e);
            toast.error("Failed to toggle playback");
        }
    }, []);

    const stop = useCallback(async () => {
        try {
            await invoke("song_pause"); // No explicit stop in backend?
            setIsPlaying(false);
            setProgress(0);
            playingSongIdRef.current = null;
        } catch (e) {
            console.error(e);
        }
    }, []);

    const seekForward = useCallback(async (seconds: number) => {
        try {
            await invoke("song_seek_forward", { seconds });
        } catch (e) {
            console.error("Failed to seek forward", e);
        }
    }, []);

    const seekBackward = useCallback(async (seconds: number) => {
        try {
            await invoke("song_seek_backward", { seconds });
        } catch (e) {
            console.error("Failed to seek backward", e);
        }
    }, []);

    return {
        isPlaying,
        setIsPlaying, // Export setter for manual overrides if needed
        progress,
        setProgress, // Export setter
        playSong,
        toggle,
        stop,
        seekForward,
        seekBackward,
    };
}
