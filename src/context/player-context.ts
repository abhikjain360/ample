import { createContext } from "react";
import { SongData } from "@/types";

export interface PlayerStateContextType {
    queue: SongData[];
    currentIndex: number;
    currentSong: SongData | null;
    isPlaying: boolean;
    isRepeating: boolean;

    play: (song: SongData, newQueue?: SongData[]) => Promise<void>;
    togglePlay: () => Promise<void>;
    toggleRepeat: () => void;
    playNext: () => Promise<void>;
    playPrev: () => Promise<void>;
    seekForward: (seconds: number) => Promise<void>;
    seekBackward: (seconds: number) => Promise<void>;
    addToQueue: (songs: SongData[]) => void;
    removeFromQueue: (index: number) => Promise<void>;
    clearQueue: () => void;
    shuffleQueue: () => void;
    shuffleAndPlay: (songs: SongData[]) => Promise<void>;
    moveInQueue: (fromIndex: number, toIndex: number) => void;
    setQueue: (songs: SongData[]) => void;
    setCurrentIndex: (index: number) => void;
}

export const PlayerStateContext = createContext<PlayerStateContextType | null>(
    null,
);
export const PlayerProgressContext = createContext<number>(0);
