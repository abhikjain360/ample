import { Play, Pause, Repeat, SkipBack, SkipForward } from "lucide-react";
import { usePlayer, usePlayerProgress } from "@/hooks/usePlayer";
import { useCallback } from "react";

export function Player() {
    const {
        currentSong: song,
        isPlaying,
        togglePlay,
        isRepeating,
        toggleRepeat,
        playNext,
        playPrev,
        seekForward,
        seekBackward,
    } = usePlayer();
    const progress = usePlayerProgress();

    const handleSeek = useCallback(
        (e: React.MouseEvent<HTMLDivElement>) => {
            if (!song) return;
            const rect = e.currentTarget.getBoundingClientRect();
            const clickX = e.clientX - rect.left;
            const ratio = Math.max(0, Math.min(1, clickX / rect.width));
            const [totalMinutes, totalSeconds] = song.duration;
            const totalDurationInSeconds = totalMinutes * 60 + totalSeconds;
            const targetSeconds = totalDurationInSeconds * ratio;
            const currentSeconds = totalDurationInSeconds * progress;
            const diff = targetSeconds - currentSeconds;
            if (diff > 0) {
                seekForward(diff);
            } else if (diff < 0) {
                seekBackward(-diff);
            }
        },
        [song, progress, seekForward, seekBackward],
    );

    if (!song) return null;

    const [totalMinutes, totalSeconds] = song.duration;
    const totalDurationInSeconds = totalMinutes * 60 + totalSeconds;
    const currentDurationInSeconds = totalDurationInSeconds * progress;

    const currentMinutes = Math.floor(currentDurationInSeconds / 60);
    const currentSeconds = Math.floor(currentDurationInSeconds % 60);

    const formatTime = (min: number, sec: number) =>
        `${min}:${sec.toString().padStart(2, "0")}`;

    return (
        <div className="bg-background/80 backdrop-blur-lg border-t border-border p-3 sm:p-4 flex items-center justify-between gap-3 sm:gap-4 transition-all duration-300 ease-in-out animate-in slide-in-from-bottom-10 relative">
            {/* Progress bar with click-to-seek */}
            <div
                className="absolute top-0 left-0 right-0 h-2 sm:h-1 bg-secondary cursor-pointer group/progress"
                onClick={handleSeek}
                role="slider"
                aria-label="Seek"
                aria-valuenow={Math.round(progress * 100)}
                aria-valuemin={0}
                aria-valuemax={100}
                tabIndex={0}
                onKeyDown={(e) => {
                    if (e.key === "ArrowLeft") seekBackward(5);
                    if (e.key === "ArrowRight") seekForward(5);
                }}
            >
                <div
                    className="h-full bg-primary transition-all duration-300 ease-linear group-hover/progress:bg-primary/90"
                    style={{ width: `${progress * 100}%` }}
                />
            </div>

            <div className="flex-1 min-w-0">
                <h3 className="font-medium truncate text-sm sm:text-base">
                    {song.title}
                </h3>
                <p className="text-xs text-muted-foreground truncate">
                    {song.artist || "Unknown Artist"}
                </p>
            </div>

            <div className="flex flex-col items-center gap-1 sm:gap-2 flex-1">
                <div className="flex items-center gap-3 sm:gap-4">
                    <button
                        onClick={playPrev}
                        className="h-8 w-8 sm:h-9 sm:w-9 rounded-full flex items-center justify-center text-foreground hover:bg-accent transition-colors"
                        title="Previous (p)"
                        aria-label="Previous song"
                    >
                        <SkipBack className="h-4 w-4 fill-current" />
                    </button>
                    <button
                        onClick={togglePlay}
                        className="h-10 w-10 sm:h-12 sm:w-12 rounded-full bg-primary text-primary-foreground flex items-center justify-center hover:bg-primary/90 transition-colors shadow-sm"
                        aria-label={isPlaying ? "Pause" : "Play"}
                    >
                        {isPlaying ? (
                            <Pause className="h-5 w-5 fill-current" />
                        ) : (
                            <Play className="h-5 w-5 fill-current ml-0.5" />
                        )}
                    </button>
                    <button
                        onClick={playNext}
                        className="h-8 w-8 sm:h-9 sm:w-9 rounded-full flex items-center justify-center text-foreground hover:bg-accent transition-colors"
                        title="Next (n)"
                        aria-label="Next song"
                    >
                        <SkipForward className="h-4 w-4 fill-current" />
                    </button>
                </div>
            </div>

            <div className="flex-1 flex items-center justify-end gap-2 sm:gap-3 text-xs font-mono text-muted-foreground">
                <button
                    onClick={toggleRepeat}
                    className={`transition-all duration-200 p-1 rounded-full hover:bg-accent ${
                        isRepeating
                            ? "text-chart-peach opacity-100 drop-shadow-[0_0_8px_rgba(250,179,135,0.5)]"
                            : "text-muted-foreground opacity-20 hover:opacity-50"
                    }`}
                    title="Toggle Repeat (r)"
                    aria-label="Toggle repeat"
                >
                    <Repeat className="h-4 w-4" />
                </button>
                <div className="hidden sm:flex gap-3">
                    <span>{formatTime(currentMinutes, currentSeconds)}</span>
                    <span>/</span>
                    <span>{formatTime(totalMinutes, totalSeconds)}</span>
                </div>
                {/* Mobile: just show current time */}
                <span className="sm:hidden">
                    {formatTime(currentMinutes, currentSeconds)}
                </span>
            </div>
        </div>
    );
}
