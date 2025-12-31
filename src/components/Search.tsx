import { useState, useMemo, useEffect, useRef, useCallback } from "react";
import Fuse from "fuse.js";
import { TableVirtuoso, VirtuosoHandle } from "react-virtuoso";
import { Clock, Play, Music } from "lucide-react";
import { SongData } from "@/types";
import { usePlayer } from "@/context/PlayerContext";
import { useVim, useVimNavigation } from "@/hooks/useVim";

interface SearchProps {
    songs: SongData[];
    onClose: () => void;
}

export default function Search({ songs, onClose }: SearchProps) {
    const [query, setQuery] = useState("");
    const [selectedIndex, setSelectedIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const virtuosoRef = useRef<VirtuosoHandle>(null);
    const { play, togglePlay, currentSong, addToQueue, shuffleAndPlay } =
        usePlayer();

    // Initialize Fuse
    const fuse = useMemo(() => {
        return new Fuse(songs, {
            keys: ["title", "artist"],
            threshold: 0.4, // Adjust for fuzziness
            ignoreLocation: true, // Search anywhere in the string
        });
    }, [songs]);

    // Derived filtered songs
    const results = useMemo(() => {
        if (!query) return songs;
        return fuse.search(query).map((result) => result.item);
    }, [songs, query, fuse]);

    // Reset selection when results change
    useEffect(() => {
        setSelectedIndex(0);
        virtuosoRef.current?.scrollToIndex(0);
    }, [results]);

    // Auto-focus input
    useEffect(() => {
        inputRef.current?.focus();
    }, []);

    const formatDuration = (duration: [number, number]) => {
        const [minutes, seconds] = duration;
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    };

    const nav = useVimNavigation(results, {
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

    const handlePlay = useCallback(async () => {
        if (results.length === 0) return;
        const song = results[selectedIndex];
        if (!song) return;
        await play(song, results);
        onClose();
    }, [results, selectedIndex, play, onClose]);

    const handleAddSelectionToQueue = useCallback(() => {
        if (results.length === 0) return;
        const song = results[selectedIndex];
        if (song) {
            addToQueue([song]);
        }
    }, [results, selectedIndex, addToQueue]);

    const handleShuffleAndPlay = useCallback(async () => {
        if (results.length === 0) return;
        await shuffleAndPlay(results);
        onClose();
    }, [results, shuffleAndPlay, onClose]);

    const bindings = useMemo(
        () => [
            {
                keys: "j",
                action: () => {
                    const idx = nav.next();
                    updateSelection(idx, "down");
                },
                when: () => results.length > 0,
            },
            {
                keys: "k",
                action: () => {
                    const idx = nav.prev();
                    updateSelection(idx, "up");
                },
                when: () => results.length > 0,
            },
            {
                keys: "gg",
                action: () => {
                    const idx = nav.first();
                    updateSelection(idx, "auto");
                },
                when: () => results.length > 0,
            },
            {
                keys: "G",
                action: () => {
                    const idx = nav.last();
                    updateSelection(idx, "auto");
                },
                when: () => results.length > 0,
            },
            {
                keys: "Enter",
                action: handlePlay,
                when: () => results.length > 0,
                noRepeat: true,
            },
            {
                keys: "a",
                action: handleAddSelectionToQueue,
                when: () => results.length > 0,
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
                when: () => results.length > 0,
                noRepeat: true,
            },
            {
                keys: ["ZZ", "Escape"],
                action: onClose,
                noRepeat: true,
            },
            {
                keys: "/",
                action: () => {
                    inputRef.current?.focus();
                },
                noRepeat: true,
            },
        ],
        [
            nav,
            results.length,
            updateSelection,
            togglePlay,
            handlePlay,
            handleAddSelectionToQueue,
            handleShuffleAndPlay,
            currentSong,
            onClose,
        ],
    );

    useVim({ bindings });

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Escape") {
            e.preventDefault();
            inputRef.current?.blur();
            return;
        }

        if (e.key === "Enter") {
            e.preventDefault();
            handlePlay();
            return;
        }
    };

    return (
        <div className="fixed inset-0 z-50 flex flex-col bg-background/95 backdrop-blur-xl text-foreground">
            {/* Search Input Area */}
            <div className="p-4 border-b border-border">
                <input
                    ref={inputRef}
                    type="text"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder="Search..."
                    className="w-full bg-transparent text-xl font-medium outline-none placeholder:text-muted-foreground"
                    autoComplete="off"
                    autoCorrect="off"
                    spellCheck="false"
                />
            </div>

            {/* Results List */}
            <div className="flex-1 overflow-hidden">
                {results.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground space-y-2">
                        <Music className="h-12 w-12 opacity-20" />
                        <p>No results found.</p>
                    </div>
                ) : (
                    <TableVirtuoso
                        ref={virtuosoRef}
                        data={results}
                        overscan={20}
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
                                    currentSong?.id === results[index].id;
                                // eslint-disable-next-line @typescript-eslint/no-unused-vars
                                const { item: _item, ...rest } = props;

                                let className =
                                    "cursor-pointer select-none border-l-2 ";

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
                                        onClick={() => {
                                            setSelectedIndex(index);
                                            play(results[index], results);
                                            onClose();
                                        }}
                                        onMouseEnter={() =>
                                            setSelectedIndex(index)
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
