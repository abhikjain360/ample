import {
    useState,
    useEffect,
    useCallback,
    useMemo,
    ReactNode,
    useRef,
} from "react";
import { SongData } from "@/types";
import { shuffle } from "@/lib/utils";
import { useSongPlayer } from "@/hooks/useSongPlayer";
import {
    PlayerStateContext,
    PlayerProgressContext,
} from "@/context/player-context";

export function PlayerProvider({ children }: { children: ReactNode }) {
    const [queue, setQueueState] = useState<SongData[]>([]);
    const [currentIndex, setCurrentIndexState] = useState<number>(-1);
    const [isRepeating, setIsRepeating] = useState(false);

    // Stable state ref for internal logic to avoid closures
    const stateRef = useRef({ queue, currentIndex, isRepeating });
    useEffect(() => {
        stateRef.current = { queue, currentIndex, isRepeating };
    }, [queue, currentIndex, isRepeating]);

    // Forward declaration for the cyclic dependency
    const playNextInternalRef = useRef<() => Promise<void>>(async () => {});

    // Initialize the low-level player
    const {
        isPlaying,
        progress,
        playSong,
        toggle,
        stop,
        seekForward,
        seekBackward,
    } = useSongPlayer(() => playNextInternalRef.current());

    // --- Actions ---

    const playNextInternal = useCallback(async () => {
        const { queue, currentIndex, isRepeating } = stateRef.current;
        if (queue.length === 0) return;

        let nextIndex = currentIndex + 1;
        if (nextIndex >= queue.length) {
            if (isRepeating) {
                nextIndex = 0;
            } else {
                // End of queue
                await stop();
                setCurrentIndexState(-1);
                return;
            }
        }

        setCurrentIndexState(nextIndex);
        const song = queue[nextIndex];
        await playSong(song);
    }, [playSong, stop]);

    const playPrevInternal = useCallback(async () => {
        const { queue, currentIndex } = stateRef.current;
        if (queue.length === 0) return;

        const prevIndex = currentIndex - 1;
        if (prevIndex >= 0) {
            setCurrentIndexState(prevIndex);
            const song = queue[prevIndex];
            await playSong(song);
        } else {
            // Restart current or stop
            if (currentIndex === 0) {
                const song = queue[0];
                await playSong(song);
            }
        }
    }, [playSong]);

    // Update the ref so useSongPlayer can call it
    useEffect(() => {
        playNextInternalRef.current = playNextInternal;
    }, [playNextInternal]);

    const play = useCallback(
        async (song: SongData, newQueue?: SongData[]) => {
            let newIndex = stateRef.current.currentIndex;
            let activeQueue = stateRef.current.queue;

            if (newQueue) {
                activeQueue = newQueue;
                setQueueState(newQueue);
                newIndex = newQueue.findIndex((s) => s.id === song.id);
                if (newIndex === -1) newIndex = 0;
            } else {
                // If not replacing queue, find in existing or add
                const idx = activeQueue.findIndex((s) => s.id === song.id);
                if (idx !== -1) {
                    newIndex = idx;
                } else {
                    activeQueue = [...activeQueue, song];
                    setQueueState(activeQueue);
                    newIndex = activeQueue.length - 1;
                }
            }

            setCurrentIndexState(newIndex);
            // We must update ref immediately for the synchronous logic following if any?
            // But we can just pass the song directly.
            await playSong(song);
        },
        [playSong],
    );

    const togglePlay = useCallback(async () => {
        // We rely on the hook's isPlaying state
        await toggle(!isPlaying);
    }, [toggle, isPlaying]);

    const toggleRepeat = useCallback(() => {
        setIsRepeating((prev) => !prev);
    }, []);

    const addToQueue = useCallback((songs: SongData[]) => {
        setQueueState((prev) => [...prev, ...songs]);
    }, []);

    const removeFromQueue = useCallback(
        async (index: number) => {
            const { queue, currentIndex } = stateRef.current;
            const newQueue = [...queue];
            newQueue.splice(index, 1);

            setQueueState(newQueue);

            if (index < currentIndex) {
                setCurrentIndexState((c) => c - 1);
            } else if (index === currentIndex) {
                if (newQueue.length === 0) {
                    await stop();
                    setCurrentIndexState(-1);
                } else {
                    let nextIndex = index;
                    if (nextIndex >= newQueue.length) {
                        nextIndex = newQueue.length - 1;
                    }
                    setCurrentIndexState(nextIndex);
                    await playSong(newQueue[nextIndex]);
                }
            }
        },
        [stop, playSong],
    );

    const clearQueue = useCallback(() => {
        setQueueState([]);
        setCurrentIndexState(-1);
        stop();
    }, [stop]);

    const shuffleQueue = useCallback(() => {
        setQueueState((prev) => {
            const currentIdx = stateRef.current.currentIndex;
            const currentSong = prev[currentIdx];

            // If playing, keep current song at current index (or 0) and shuffle rest?
            // Common behavior: Shuffle everything, but put current song at top?
            // Or just shuffle.

            if (!currentSong) return shuffle(prev);

            // Let's shuffle and find new index
            const shuffled = shuffle(prev);
            const newIndex = shuffled.findIndex((s) => s.id === currentSong.id);
            setCurrentIndexState(newIndex);
            return shuffled;
        });
    }, []);

    const moveInQueue = useCallback((fromIndex: number, toIndex: number) => {
        setQueueState((prev) => {
            if (
                fromIndex < 0 ||
                fromIndex >= prev.length ||
                toIndex < 0 ||
                toIndex >= prev.length ||
                fromIndex === toIndex
            ) {
                return prev;
            }

            const newQueue = [...prev];
            const [movedItem] = newQueue.splice(fromIndex, 1);
            newQueue.splice(toIndex, 0, movedItem);

            const currentIdx = stateRef.current.currentIndex;
            if (fromIndex === currentIdx) {
                setCurrentIndexState(toIndex);
            } else {
                if (fromIndex < currentIdx && toIndex >= currentIdx) {
                    setCurrentIndexState((c) => c - 1);
                } else if (fromIndex > currentIdx && toIndex <= currentIdx) {
                    setCurrentIndexState((c) => c + 1);
                }
            }
            return newQueue;
        });
    }, []);

    const shuffleAndPlay = useCallback(
        async (songs: SongData[]) => {
            const shuffled = shuffle(songs);
            if (shuffled.length === 0) return;
            await play(shuffled[0], shuffled);
        },
        [play],
    );

    const setQueue = useCallback((songs: SongData[]) => {
        setQueueState(songs);
    }, []);

    const setCurrentIndex = useCallback((index: number) => {
        setCurrentIndexState(index);
    }, []);

    const stateValue = useMemo(
        () => ({
            queue,
            currentIndex,
            currentSong: queue[currentIndex] || null,
            isPlaying,
            isRepeating,
            play,
            togglePlay,
            toggleRepeat,
            playNext: playNextInternal,
            playPrev: playPrevInternal,
            seekForward,
            seekBackward,
            addToQueue,
            removeFromQueue,
            clearQueue,
            shuffleQueue,
            moveInQueue,
            shuffleAndPlay,
            setQueue,
            setCurrentIndex,
        }),
        [
            queue,
            currentIndex,
            isPlaying,
            isRepeating,
            play,
            togglePlay,
            toggleRepeat,
            playNextInternal,
            playPrevInternal,
            seekForward,
            seekBackward,
            addToQueue,
            removeFromQueue,
            clearQueue,
            shuffleQueue,
            moveInQueue,
            shuffleAndPlay,
            setQueue,
            setCurrentIndex,
        ],
    );

    return (
        <PlayerStateContext.Provider value={stateValue}>
            <PlayerProgressContext.Provider value={progress}>
                {children}
            </PlayerProgressContext.Provider>
        </PlayerStateContext.Provider>
    );
}
