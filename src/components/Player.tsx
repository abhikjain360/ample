import { Play, Pause } from "lucide-react";
import { usePlayer, usePlayerProgress } from "@/context/PlayerContext";

export function Player() {
    const { currentSong: song, isPlaying, togglePlay } = usePlayer();
    const progress = usePlayerProgress();

    if (!song) return null;

    const [totalMinutes, totalSeconds] = song.duration;
    const totalDurationInSeconds = totalMinutes * 60 + totalSeconds;
    const currentDurationInSeconds = totalDurationInSeconds * progress;

    const currentMinutes = Math.floor(currentDurationInSeconds / 60);
    const currentSeconds = Math.floor(currentDurationInSeconds % 60);

    const formatTime = (min: number, sec: number) =>
        `${min}:${sec.toString().padStart(2, "0")}`;

    return (
        <div className="bg-background/80 backdrop-blur-lg border-t border-border p-4 flex items-center justify-between gap-4 transition-all duration-300 ease-in-out animate-in slide-in-from-bottom-10 relative">
            <div className="absolute top-0 left-0 right-0 h-1 bg-secondary">
                <div
                    className="h-full bg-primary transition-all duration-300 ease-linear"
                    style={{ width: `${progress * 100}%` }}
                />
            </div>

            <div className="flex-1 min-w-0">
                <h3 className="font-medium truncate">{song.title}</h3>
                <p className="text-xs text-muted-foreground truncate">
                    {song.artist || "Unknown Artist"}
                </p>
            </div>

            <div className="flex flex-col items-center gap-2 flex-1">
                <div className="flex items-center gap-4">
                    <button
                        onClick={togglePlay}
                        className="h-10 w-10 rounded-full bg-primary text-primary-foreground flex items-center justify-center hover:bg-primary/90 transition-colors"
                    >
                        {isPlaying ? (
                            <Pause className="h-5 w-5 fill-current" />
                        ) : (
                            <Play className="h-5 w-5 fill-current ml-0.5" />
                        )}
                    </button>
                </div>
            </div>

            <div className="flex-1 flex items-center justify-end gap-3 text-xs font-mono text-muted-foreground">
                <span>{formatTime(currentMinutes, currentSeconds)}</span>
                <span>/</span>
                <span>{formatTime(totalMinutes, totalSeconds)}</span>
            </div>
        </div>
    );
}
