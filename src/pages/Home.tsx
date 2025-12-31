import { useEffect, useState, useMemo, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Play, Clock, Music } from "lucide-react";
import { toast } from "sonner";
import { useLocation } from "wouter";
import { TableVirtuoso, VirtuosoHandle } from "react-virtuoso";
import Loading from "@/components/Loading";
import { useVim, useVimNavigation } from "@/hooks/useVim";
import { SongData } from "@/types";
import { usePlayer } from "@/context/PlayerContext";

export default function Home() {
    const [songs, setSongs] = useState<SongData[] | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [selectedIndex, setSelectedIndex] = useState(0);

    const { currentSong, togglePlay, play, addToQueue, shuffleAndPlay } =
        usePlayer();

    const [, setLocation] = useLocation();
    const virtuosoRef = useRef<VirtuosoHandle>(null);

    useEffect(() => {
        fetchSongs();
    }, []);

    useEffect(() => {
        if (currentSong && virtuosoRef.current) {
            // We don't necessarily want to scroll to current song on Home page if user is browsing elsewhere.
            // The original code did this, but now with queue, the playing song might not even be on this list (if we navigated away and back?).
            // But if it IS in the list, maybe we only scroll if we just started playing it?
            // Let's disable auto-scroll to playing song on Home for now, as it might disrupt browsing.
            // User can press 'Space' or navigation keys to move.
        }
    }, [currentSong]);

    const fetchSongs = async () => {
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
    };

    const formatDuration = (duration: [number, number]) => {
        const [minutes, seconds] = duration;
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    };

    const nav = useVimNavigation(songs ?? [], {
        onSelect: () => {
            // handled by bindings
        },
    });

    const updateSelection = useCallback(
        (newIndex: number, direction: "up" | "down" | "auto" = "auto") => {
            setSelectedIndex(newIndex);

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
        [],
    );

    const handleAddSelectionToQueue = useCallback(() => {
        if (!songs) return;
        const song = songs[selectedIndex];
        if (song) {
            addToQueue([song]);
            toast.success(`Added "${song.title}" to queue`);
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
                keys: "s",
                action: togglePlay,
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
                    <TableVirtuoso
                        ref={virtuosoRef}
                        data={songs}
                        overscan={200}
                        className="h-full w-full scroll-pt-11"
                        fixedHeaderContent={() => (
                            <tr className="bg-muted/70 backdrop-blur-md border-b-2 border-border text-xs text-muted-foreground uppercase">
                                <th className="px-4 py-3 font-medium w-12 text-center bg-muted/70 backdrop-blur-md">
                                    #
                                </th>
                                <th className="px-4 py-3 font-medium bg-muted/70 backdrop-blur-md text-left">
                                    Title
                                </th>
                                <th className="px-4 py-3 font-medium bg-muted/70 backdrop-blur-md text-left">
                                    Artist
                                </th>
                                <th className="px-4 py-3 font-medium text-right w-24 bg-muted/70 backdrop-blur-md">
                                    <Clock className="h-3 w-3 inline-block" />
                                </th>
                            </tr>
                        )}
                        itemContent={(index, song) => {
                            const isPlayingCurrent =
                                currentSong?.id === song.id;

                            // Text color logic
                            const textColorClass = isPlayingCurrent
                                ? "text-green-500"
                                : "text-foreground";
                            const mutedTextClass = isPlayingCurrent
                                ? "text-green-500/70"
                                : "text-muted-foreground";

                            return (
                                <>
                                    <td
                                        className={`px-4 py-3 text-center font-mono text-xs w-12 ${textColorClass}`}
                                    >
                                        {isPlayingCurrent ? (
                                            <Play className="h-3 w-3 mx-auto fill-current" />
                                        ) : (
                                            <span className="opacity-50">
                                                {index + 1}
                                            </span>
                                        )}
                                    </td>
                                    <td
                                        className={`px-4 py-3 font-medium truncate max-w-[30vw] ${textColorClass}`}
                                    >
                                        {song.title}
                                    </td>
                                    <td
                                        className={`px-4 py-3 truncate max-w-[20vw] ${mutedTextClass}`}
                                    >
                                        {song.artist || "Unknown Artist"}
                                    </td>
                                    <td
                                        className={`px-4 py-3 text-right font-mono text-xs w-24 ${mutedTextClass}`}
                                    >
                                        {formatDuration(song.duration)}
                                    </td>
                                </>
                            );
                        }}
                        components={{
                            Table: (props) => (
                                <table
                                    {...props}
                                    className="w-full table-fixed border-collapse"
                                />
                            ),
                            TableRow: (props) => {
                                const index = props["data-index"];
                                const isFocused = selectedIndex === index;
                                const isPlayingCurrent =
                                    currentSong?.id === songs[index].id;
                                // eslint-disable-next-line @typescript-eslint/no-unused-vars
                                const { item: _item, ...rest } = props;

                                let className =
                                    "group cursor-default select-none border-l-2 ";

                                // Border logic
                                if (isPlayingCurrent) {
                                    className += "border-l-green-500 ";
                                } else if (isFocused) {
                                    className += "border-l-primary ";
                                } else {
                                    className += "border-l-transparent ";
                                }

                                // Background logic
                                if (isFocused) {
                                    className += "bg-accent ";
                                } else {
                                    className += "hover:bg-accent/30 ";
                                }

                                return (
                                    <tr
                                        {...rest}
                                        className={className}
                                        onClick={() => updateSelection(index)}
                                        onDoubleClick={() =>
                                            play(songs[index], songs)
                                        }
                                    />
                                );
                            },
                        }}
                    />
                )}
            </div>
        </div>
    );
}
