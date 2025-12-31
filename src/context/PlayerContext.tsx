import {
    createContext,
    useContext,
    useState,
    useEffect,
    useCallback,
    useMemo,
    ReactNode,
    useRef,
} from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { toast } from "sonner";
import { SongData, PlaybackPayload } from "@/types";
import { shuffle } from "@/lib/utils";

interface PlayerStateContextType {
    queue: SongData[];
    currentIndex: number;
    currentSong: SongData | null;
    isPlaying: boolean;
    
    play: (song: SongData, newQueue?: SongData[]) => Promise<void>;
    togglePlay: () => Promise<void>;
    playNext: () => Promise<void>;
    playPrev: () => Promise<void>;
    addToQueue: (songs: SongData[]) => void;
    removeFromQueue: (index: number) => void;
    clearQueue: () => void;
    shuffleQueue: () => void;
    shuffleAndPlay: (songs: SongData[]) => Promise<void>;
    moveInQueue: (fromIndex: number, toIndex: number) => void;
    setQueue: (songs: SongData[]) => void;
    setCurrentIndex: (index: number) => void;
}

const PlayerStateContext = createContext<PlayerStateContextType | null>(null);
const PlayerProgressContext = createContext<number>(0);

export function PlayerProvider({ children }: { children: ReactNode }) {
    const [queue, setQueueState] = useState<SongData[]>([]);
    const [currentIndex, setCurrentIndexState] = useState<number>(-1);
    const [isPlaying, setIsPlaying] = useState(false);
    const [progress, setProgress] = useState(0);

    const currentSong = queue[currentIndex] || null;
    
    // We need to keep track of the current song ID being played to avoid race conditions
    // or processing events for old songs.
    const playingSongIdRef = useRef<number | null>(null);

    const setQueue = useCallback((songs: SongData[]) => {
        setQueueState(songs);
    }, []);

    const setCurrentIndex = useCallback((index: number) => {
        setCurrentIndexState(index);
    }, []);

    const playInternal = async (song: SongData) => {
        try {
            // Stop any existing playback first? miniaudio backend likely handles it, 
            // but let's reset local state.
            setIsPlaying(true);
            setProgress(0);
            playingSongIdRef.current = song.id;

            const onEvent = new Channel<PlaybackPayload>();
            onEvent.onmessage = (payload) => {
                // If we've switched songs, ignore events from the old one
                if (playingSongIdRef.current !== song.id) return;

                const { progress_frames, total_frames, is_finished } = payload;
                if (is_finished) {
                    // Auto play next
                    // To avoid closure issues, we should call playNext but we can't easily here.
                    // Instead we can use a ref to a function or rely on a different mechanism.
                    // But playNextInternal uses a Ref for state, so it IS safe to call from here?
                    // NO, because `playNextInternal` is defined below. 
                    // However, we can use a Mutable Ref that holds the latest `playNextInternal`.
                    // Or since we redefined `playInternal` inside `playSongAndSetupListeners` which IS
                    // rebuilt when `playNextInternal` changes... wait.
                    
                    // Actually, the easiest way is to use a Ref for playNextInternal.
                    playNextInternalRef.current();
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
    };
    
    // Ref to hold the latest playNextInternal so onEvent can call it
    const playNextInternalRef = useRef<() => Promise<void>>(async () => {});

    // To solve the closure stale state issue:
    const stateRef = useRef({ queue, currentIndex, isPlaying });
    useEffect(() => {
        stateRef.current = { queue, currentIndex, isPlaying };
    }, [queue, currentIndex, isPlaying]);

    const playNextInternal = useCallback(async () => {
        const { queue, currentIndex } = stateRef.current;
        if (queue.length === 0) return;

        const nextIndex = currentIndex + 1;
        if (nextIndex < queue.length) {
            setCurrentIndexState(nextIndex);
            
            // We need to play the new song.
            // But playInternal needs to be robust.
            // Let's call our action wrapper
            // But playAction calls playNextInternal, cycle?
            // No, playAction calls playSongAndSetupListeners.
            
            const song = queue[nextIndex];
            
            // We duplicate the play logic here slightly or extract it?
            // Let's call the low level player directly
            try {
                setIsPlaying(true);
                setProgress(0);
                playingSongIdRef.current = song.id;

                const onEvent = new Channel<PlaybackPayload>();
                onEvent.onmessage = (payload) => {
                    if (playingSongIdRef.current !== song.id) return;

                    const { progress_frames, total_frames, is_finished } = payload;
                    if (is_finished) {
                        playNextInternalRef.current();
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
                setIsPlaying(false);
            }

        } else {
            // End of queue
            setIsPlaying(false);
            setProgress(0);
            playingSongIdRef.current = null; // Reset
        }
    }, []);
    
    // Update the ref
    useEffect(() => {
        playNextInternalRef.current = playNextInternal;
    }, [playNextInternal]);

    const playPrevInternal = useCallback(async () => {
        const { queue, currentIndex } = stateRef.current;
        if (queue.length === 0) return;

        const prevIndex = currentIndex - 1;
        if (prevIndex >= 0) {
            setCurrentIndexState(prevIndex);
            
            // Duplicate play logic again? Ideally extract "playCore"
            const song = queue[prevIndex];
             try {
                setIsPlaying(true);
                setProgress(0);
                playingSongIdRef.current = song.id;

                const onEvent = new Channel<PlaybackPayload>();
                onEvent.onmessage = (payload) => {
                    if (playingSongIdRef.current !== song.id) return;

                    const { progress_frames, total_frames, is_finished } = payload;
                    if (is_finished) {
                        playNextInternalRef.current();
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
                setIsPlaying(false);
            }
        } else {
             // Restart current song
             if (queue.length > 0) {
                 setCurrentIndexState(0);
                 const song = queue[0];
                 // Play core...
                  try {
                    setIsPlaying(true);
                    setProgress(0);
                    playingSongIdRef.current = song.id;

                    const onEvent = new Channel<PlaybackPayload>();
                    onEvent.onmessage = (payload) => {
                        if (playingSongIdRef.current !== song.id) return;

                        const { progress_frames, total_frames, is_finished } = payload;
                        if (is_finished) {
                            playNextInternalRef.current();
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
                    setIsPlaying(false);
                }
             }
        }
    }, []);

    // Now expose these as stable functions that update state
    const togglePlay = useCallback(async () => {
        const { currentSong, isPlaying } = stateRef.current;
        if (!currentSong) return;

        try {
            if (isPlaying) {
                await invoke("song_pause");
                setIsPlaying(false);
            } else {
                await invoke("song_play");
                setIsPlaying(true);
            }
        } catch (e) {
            console.error(e);
            toast.error("Failed to toggle playback");
        }
    }, []);
    
    // playCore helper to avoid duplication
    const playCore = async (song: SongData) => {
         try {
            setIsPlaying(true);
            setProgress(0);
            playingSongIdRef.current = song.id;

            const onEvent = new Channel<PlaybackPayload>();
            onEvent.onmessage = (payload) => {
                if (playingSongIdRef.current !== song.id) return;

                const { progress_frames, total_frames, is_finished } = payload;
                if (is_finished) {
                    playNextInternalRef.current();
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
    };

    const playAction = useCallback(async (song: SongData, newQueue?: SongData[]) => {
        let newIndex = stateRef.current.currentIndex;
        let activeQueue = stateRef.current.queue;

        if (newQueue) {
            activeQueue = newQueue;
            setQueueState(newQueue);
            newIndex = newQueue.findIndex(s => s.id === song.id);
            if (newIndex === -1) newIndex = 0;
        } else {
             const idx = activeQueue.findIndex(s => s.id === song.id);
             if (idx !== -1) {
                 newIndex = idx;
             } else {
                 activeQueue = [...activeQueue, song];
                 setQueueState(activeQueue);
                 newIndex = activeQueue.length - 1;
             }
        }

        setCurrentIndexState(newIndex);
        stateRef.current = { ...stateRef.current, queue: activeQueue, currentIndex: newIndex };
        
        await playCore(song);
    }, []); // playCore is stable enough or we can depend on it if we move it inside/usecallback

    const addToQueue = useCallback((songs: SongData[]) => {
        setQueueState(prev => [...prev, ...songs]);
    }, []);

    const removeFromQueue = useCallback((index: number) => {
        setQueueState(prev => {
            const newQueue = [...prev];
            newQueue.splice(index, 1);
            
            if (index < stateRef.current.currentIndex) {
                setCurrentIndexState(c => c - 1);
            } else if (index === stateRef.current.currentIndex) {
                if (newQueue.length === 0) {
                     invoke("song_pause").catch(console.error);
                     setIsPlaying(false);
                     setCurrentIndexState(-1);
                     playingSongIdRef.current = null;
                } else {
                    if (index >= newQueue.length) {
                         setCurrentIndexState(newQueue.length - 1);
                    }
                }
            }
            return newQueue;
        });
    }, []);

    const clearQueue = useCallback(() => {
        setQueueState([]);
        setCurrentIndexState(-1);
        setIsPlaying(false);
        setProgress(0);
        playingSongIdRef.current = null;
        invoke("song_pause").catch(console.error);
    }, []);

    const shuffleQueue = useCallback(() => {
        setQueueState(prev => {
             const currentSong = prev[stateRef.current.currentIndex];
             if (!currentSong) return shuffle(prev);

             const shuffled = shuffle(prev);
             const newIndex = shuffled.findIndex(s => s.id === currentSong.id);
             setCurrentIndexState(newIndex);
             
             return shuffled;
        });
    }, []);

    const moveInQueue = useCallback((fromIndex: number, toIndex: number) => {
        setQueueState(prev => {
            if (fromIndex < 0 || fromIndex >= prev.length || toIndex < 0 || toIndex >= prev.length) return prev;
            if (fromIndex === toIndex) return prev;

            const newQueue = [...prev];
            const [movedItem] = newQueue.splice(fromIndex, 1);
            newQueue.splice(toIndex, 0, movedItem);
            
            const currentIndex = stateRef.current.currentIndex;

            if (fromIndex === currentIndex) {
                setCurrentIndexState(toIndex);
            } else {
                if (fromIndex < currentIndex && toIndex >= currentIndex) {
                    setCurrentIndexState(c => c - 1);
                } else if (fromIndex > currentIndex && toIndex <= currentIndex) {
                    setCurrentIndexState(c => c + 1);
                }
            }

            return newQueue;
        });
    }, []);

    const shuffleAndPlay = useCallback(async (songs: SongData[]) => {
        const shuffled = shuffle(songs);
        if (shuffled.length === 0) return;
        
        await playAction(shuffled[0], shuffled);
    }, [playAction]);

    const stateValue = useMemo(() => ({
        queue,
        currentIndex,
        currentSong,
        isPlaying,
        play: playAction,
        togglePlay,
        playNext: playNextInternal,
        playPrev: playPrevInternal,
        addToQueue,
        removeFromQueue,
        clearQueue,
        shuffleQueue,
        moveInQueue,
        shuffleAndPlay,
        setQueue,
        setCurrentIndex,
    }), [
        queue,
        currentIndex,
        currentSong,
        isPlaying,
        playAction,
        togglePlay,
        playNextInternal,
        playPrevInternal,
        addToQueue,
        removeFromQueue,
        clearQueue,
        shuffleQueue,
        moveInQueue,
        shuffleAndPlay,
        setQueue,
        setCurrentIndex
    ]);

    return (
        <PlayerStateContext.Provider value={stateValue}>
            <PlayerProgressContext.Provider value={progress}>
                {children}
            </PlayerProgressContext.Provider>
        </PlayerStateContext.Provider>
    );
}

export function usePlayer() {
    const context = useContext(PlayerStateContext);
    if (!context) {
        throw new Error("usePlayer must be used within a PlayerProvider");
    }
    return context;
}

export function usePlayerProgress() {
    const context = useContext(PlayerProgressContext);
    // context can be 0, which is valid.
    if (context === undefined) {
         throw new Error("usePlayerProgress must be used within a PlayerProvider");
    }
    return context;
}
