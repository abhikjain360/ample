import { useState, useMemo, useEffect, useRef, useCallback } from "react";
import Fuse from "fuse.js";
import { VirtuosoHandle } from "react-virtuoso";
import { invoke } from "@tauri-apps/api/core";
import { Music } from "lucide-react";
import { useLocation } from "wouter";
import { toast } from "sonner";
import { SongData } from "@/types";
import { usePlayer } from "@/hooks/usePlayer";
import { useVim, useVimNavigation } from "@/hooks/useVim";
import Loading from "@/components/Loading";
import SongList from "@/components/SongList";

export default function SearchPage() {
    const [songs, setSongs] = useState<SongData[] | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [query, setQuery] = useState("");
    const [selectedIndex, setSelectedIndex] = useState(0);
    const inputRef = useRef<HTMLInputElement>(null);
    const virtuosoRef = useRef<VirtuosoHandle>(null);
    const { play, togglePlay, currentSong, addToQueue, shuffleAndPlay } =
        usePlayer();
    const [, setLocation] = useLocation();

    // Fetch songs
    useEffect(() => {
        const fetchSongs = async () => {
            try {
                const songList = await invoke<SongData[]>("library_list_songs");
                setSongs(songList);
            } catch (e) {
                console.error(e);
                toast.error("Failed to fetch songs", {
                    description: String(e),
                });
            } finally {
                setIsLoading(false);
            }
        };
        fetchSongs();
    }, []);

    // Initialize Fuse
    const fuse = useMemo(() => {
        if (!songs) return null;

        return new Fuse(songs, {
            keys: ["title", "artist"],
            threshold: 0.3,
            ignoreLocation: true,
        });
    }, [songs]);

    // Derived filtered songs
    const results = useMemo(() => {
        if (!songs) return [];
        if (!query || !fuse) return songs;
        return fuse.search(query).map((result) => result.item);
    }, [songs, query, fuse]);

    const nav = useVimNavigation(results);

    // Reset selection when results change
    useEffect(() => {
        setSelectedIndex(0);
        nav.setIndex(0);
        virtuosoRef.current?.scrollToIndex(0);
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [results]);

    // Auto-focus input
    useEffect(() => {
        if (!isLoading) {
            // Small timeout to ensure render is complete and ref is attached
            const timer = setTimeout(() => {
                inputRef.current?.focus();
            }, 50);
            return () => clearTimeout(timer);
        }
    }, [isLoading]);

    const updateSelection = useCallback(
        (newIndex: number, direction: "up" | "down" | "auto" = "auto") => {
            setSelectedIndex(newIndex);
            nav.setIndex(newIndex);

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

    const handlePlay = useCallback(
        async (all: boolean = false) => {
            if (results.length === 0) return;
            const song = results[selectedIndex];
            if (!song) return;
            const songs = all ? results : [song];
            await play(song, songs);
        },
        [results, selectedIndex, play],
    );

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
    }, [results, shuffleAndPlay]);

    const goBack = useCallback(() => {
        setLocation("/home");
    }, [setLocation]);

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
                action: () => handlePlay(true),
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
                    else handlePlay(false);
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
                keys: ["Escape"],
                action: goBack,
                noRepeat: true,
            },
            {
                keys: "/",
                action: () => {
                    inputRef.current?.focus();
                    inputRef.current?.select();
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
            goBack,
        ],
    );

    useVim({ bindings });

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Escape") {
            e.preventDefault();
            if (document.activeElement === inputRef.current) {
                inputRef.current?.blur();
            } else {
                goBack();
            }
            return;
        }

        if (e.key === "Enter") {
            e.preventDefault();
            handlePlay();
            inputRef.current?.blur();
            return;
        }
    };

    return (
        <div className="h-full w-full flex flex-col bg-background text-foreground">
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
                    autoFocus
                />
            </div>

            {/* Results List */}
            <div className="flex-1 overflow-hidden">
                {isLoading ? (
                    <div className="h-full flex items-center justify-center">
                        <Loading />
                    </div>
                ) : !songs || songs.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground space-y-2">
                        <Music className="h-12 w-12 opacity-20" />
                        <p>Library is empty.</p>
                    </div>
                ) : results.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground space-y-2">
                        <Music className="h-12 w-12 opacity-20" />
                        <p>No results found.</p>
                    </div>
                ) : (
                    <SongList
                        songs={results}
                        currentSong={currentSong}
                        selectedIndex={selectedIndex}
                        onSelect={(index) => updateSelection(index, "auto")}
                        onPlay={(song) => play(song, results)}
                        virtuosoRef={virtuosoRef}
                        overscan={50}
                    />
                )}
            </div>
        </div>
    );
}
