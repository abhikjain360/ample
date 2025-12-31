import { useEffect, useState, useMemo, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Music } from "lucide-react";
import { toast } from "sonner";
import { useLocation } from "wouter";
import { VirtuosoHandle } from "react-virtuoso";
import Loading from "@/components/Loading";
import SongList from "@/components/SongList";
import { useVim, useVimNavigation } from "@/hooks/useVim";
import { SongData } from "@/types";
import { usePlayer } from "@/hooks/usePlayer";

export default function Home() {
    const [songs, setSongs] = useState<SongData[] | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedIndex, setSelectedIndex] = useState(0);

    const { currentSong, togglePlay, play, addToQueue, shuffleAndPlay } =
        usePlayer();

    const [, setLocation] = useLocation();
    const virtuosoRef = useRef<VirtuosoHandle>(null);

    const fetchSongs = useCallback(async () => {
        try {
            const songList = await invoke<SongData[]>("library_list_songs");
            setSongs(songList);
        } catch (e) {
            console.error(e);
            toast.error("Failed to fetch songs", {
                description: String(e),
            });
            setLocation("/");
        } finally {
            setIsLoading(false);
        }
    }, [setLocation]);

    useEffect(() => {
        fetchSongs();
    }, [fetchSongs]);

    const nav = useVimNavigation(songs ?? [], {
        onSelect: () => {
            // handled by bindings
        },
    });

    const updateSelection = useCallback(
        (newIndex: number, direction: "up" | "down" | "auto" = "auto") => {
            setSelectedIndex(newIndex);
            nav.setIndex(newIndex); // Ensure vim nav state is updated if set externally (mouse)

            if (direction === "auto") {
                virtuosoRef.current?.scrollIntoView({
                    index: newIndex,
                    behavior: "auto",
                    align: "center",
                });
                return;
            }

            const node = document.querySelector(`tr[data-index="${newIndex}"]`);
            if (node) {
                node.scrollIntoView({
                    behavior: "auto",
                    block: "nearest",
                });
                return;
            }

            virtuosoRef.current?.scrollIntoView({
                index: newIndex,
                behavior: "auto",
                align: direction === "down" ? "end" : "start",
            });
        },
        [nav],
    );

    const handleAddSelectionToQueue = useCallback(() => {
        if (!songs) return;
        const song = songs[selectedIndex];
        if (song) {
            addToQueue([song]);
        }
    }, [songs, selectedIndex, addToQueue]);

    const handlePlay = useCallback(async () => {
        if (!songs) return;
        const song = songs[selectedIndex];
        if (!song) return;

        await play(song, songs);
    }, [songs, selectedIndex, play]);

    const handleShuffleAndPlay = useCallback(async () => {
        if (!songs || songs.length === 0) return;
        await shuffleAndPlay(songs);
    }, [songs, shuffleAndPlay]);

    const bindings = useMemo(
        () => [
            {
                keys: "j",
                action: () => {
                    const idx = nav.next();
                    updateSelection(idx, "down");
                },
                when: () => (songs?.length ?? 0) > 0,
            },
            {
                keys: "k",
                action: () => {
                    const idx = nav.prev();
                    updateSelection(idx, "up");
                },
                when: () => (songs?.length ?? 0) > 0,
            },
            {
                keys: "gg",
                action: () => {
                    const idx = nav.first();
                    updateSelection(idx, "auto");
                },
                when: () => (songs?.length ?? 0) > 0,
            },
            {
                keys: "G",
                action: () => {
                    const idx = nav.last();
                    updateSelection(idx, "auto");
                },
                when: () => (songs?.length ?? 0) > 0,
            },
            {
                keys: "Enter",
                action: handlePlay,
                when: () => (songs?.length ?? 0) > 0,
                noRepeat: true,
            },
            {
                keys: "a",
                action: handleAddSelectionToQueue,
                when: () => (songs?.length ?? 0) > 0,
                noRepeat: true,
            },
            {
                keys: " ",
                action: () => {
                    if (currentSong) togglePlay();
                    else handlePlay();
                },
                noRepeat: true,
            },
            {
                keys: "S",
                action: handleShuffleAndPlay,
                when: () => (songs?.length ?? 0) > 0,
                noRepeat: true,
            },
            {
                keys: "ZZ",
                action: () => setLocation("/"),
                noRepeat: true,
            },
        ],
        [
            nav,
            songs?.length,
            updateSelection,
            togglePlay,
            handlePlay,
            handleAddSelectionToQueue,
            handleShuffleAndPlay,
            currentSong,
            setLocation,
        ],
    );

    useVim({ bindings });

    if (isLoading) {
        return (
            <div className="h-full flex items-center justify-center bg-background text-foreground">
                <Loading />
            </div>
        );
    }

    if (!songs) return null;

    return (
        <div className="h-full w-full bg-background text-foreground flex flex-col">
            <div className="flex-1 relative overflow-hidden">
                {songs.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground space-y-2">
                        <Music className="h-12 w-12 opacity-20" />
                        <p>No songs found in this library.</p>
                    </div>
                ) : (
                    <SongList
                        songs={songs}
                        currentSong={currentSong}
                        selectedIndex={selectedIndex}
                        onSelect={(index) => updateSelection(index, "auto")}
                        onPlay={(song) => play(song, songs)}
                        virtuosoRef={virtuosoRef}
                        overscan={200}
                    />
                )}
            </div>
        </div>
    );
}
