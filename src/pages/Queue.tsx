import { useEffect, useMemo, useCallback, useRef, useState } from "react";
import { Play, Clock, Music } from "lucide-react";
import { TableVirtuoso, VirtuosoHandle } from "react-virtuoso";
import { useVim, useVimNavigation } from "@/hooks/useVim";
import { usePlayer } from "@/context/PlayerContext";

export default function Queue() {
    const {
        queue,
        currentIndex,
        setCurrentIndex,
        play,
        togglePlay,
        removeFromQueue,
        shuffleQueue,
        moveInQueue,
    } = usePlayer();

    const [selectedIndex, setSelectedIndex] = useState(0);
    const virtuosoRef = useRef<VirtuosoHandle>(null);

    // Sync selected index with current playing song initially if possible
    useEffect(() => {
        if (currentIndex !== -1) {
            setSelectedIndex(currentIndex);
        }
    }, []);

    const formatDuration = (duration: [number, number]) => {
        const [minutes, seconds] = duration;
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    };

    const nav = useVimNavigation(queue, {
        onSelect: () => {
            // Handled by Enter binding
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

    const handlePlay = useCallback(() => {
        if (queue.length === 0) return;
        // Play the selected song from the queue
        const song = queue[selectedIndex];
        if (song) {
            play(song);
        }
    }, [queue, selectedIndex, play]);

    const handleRemove = useCallback(() => {
        if (queue.length === 0) return;
        removeFromQueue(selectedIndex);
        // If we remove the last item, select the new last item
        if (selectedIndex >= queue.length - 1) {
            setSelectedIndex(Math.max(0, queue.length - 2));
        }
    }, [queue, selectedIndex, removeFromQueue]);

    const handleMoveDown = useCallback(() => {
        if (queue.length <= 1) return;
        // Don't move if it's already at the bottom
        if (selectedIndex >= queue.length - 1) return;

        moveInQueue(selectedIndex, selectedIndex + 1);
        // Select the moved item in its new position
        updateSelection(selectedIndex + 1, "down");
    }, [queue, selectedIndex, moveInQueue, updateSelection]);

    const handleMoveUp = useCallback(() => {
        if (queue.length <= 1) return;
        // Don't move if it's already at the top
        if (selectedIndex <= 0) return;

        moveInQueue(selectedIndex, selectedIndex - 1);
        // Select the moved item in its new position
        updateSelection(selectedIndex - 1, "up");
    }, [queue, selectedIndex, moveInQueue, updateSelection]);

    const bindings = useMemo(
        () => [
            {
                keys: "j",
                action: () => {
                    const idx = nav.next();
                    updateSelection(idx, "down");
                },
                when: () => queue.length > 0,
            },
            {
                keys: "k",
                action: () => {
                    const idx = nav.prev();
                    updateSelection(idx, "up");
                },
                when: () => queue.length > 0,
            },
            {
                keys: "gg",
                action: () => {
                    const idx = nav.first();
                    updateSelection(idx, "auto");
                },
                when: () => queue.length > 0,
            },
            {
                keys: "G",
                action: () => {
                    const idx = nav.last();
                    updateSelection(idx, "auto");
                },
                when: () => queue.length > 0,
            },
            {
                keys: "N",
                action: handleMoveDown,
                when: () => queue.length > 0,
                noRepeat: true,
            },
            {
                keys: "P",
                action: handleMoveUp,
                when: () => queue.length > 0,
                noRepeat: true,
            },
            {
                keys: "Enter",
                action: handlePlay,
                when: () => queue.length > 0,
                noRepeat: true,
            },
            {
                keys: "x",
                action: handleRemove,
                when: () => queue.length > 0,
                noRepeat: true,
            },
            {
                keys: "s",
                action: togglePlay,
                noRepeat: true,
            },
            {
                keys: "S",
                action: shuffleQueue,
                when: () => queue.length > 0,
                noRepeat: true,
            },
        ],
        [
            nav,
            queue.length,
            updateSelection,
            handlePlay,
            handleRemove,
            shuffleQueue,
            togglePlay,
            handleMoveDown,
            handleMoveUp,
        ],
    );

    useVim({ bindings });

    return (
        <div className="h-full w-full bg-background text-foreground flex flex-col">
            <div className="p-4 border-b border-border bg-muted/20 backdrop-blur-sm">
                <h1 className="text-xl font-bold flex items-center gap-2">
                    <Music className="h-5 w-5" />
                    Queue ({queue.length})
                </h1>
            </div>

            <div className="flex-1 relative overflow-hidden">
                {queue.length === 0 ? (
                    <div className="h-full flex flex-col items-center justify-center text-muted-foreground space-y-2">
                        <p>Queue is empty.</p>
                        <p className="text-sm">
                            Press 'q' to go home and add songs.
                        </p>
                    </div>
                ) : (
                    <TableVirtuoso
                        ref={virtuosoRef}
                        data={queue}
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
                            // Check if this specific instance in queue is playing.
                            // Since queue can have duplicates?
                            // Our logic relies on currentIndex.
                            const isPlayingCurrent = index === currentIndex;

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
                                const isPlayingCurrent = index === currentIndex;
                                // eslint-disable-next-line @typescript-eslint/no-unused-vars
                                const { item: _item, ...rest } = props;

                                let className =
                                    "group cursor-default select-none border-l-2 ";

                                if (isPlayingCurrent) {
                                    className += "border-l-green-500 ";
                                } else if (isFocused) {
                                    className += "border-l-primary ";
                                } else {
                                    className += "border-l-transparent ";
                                }

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
                                            setCurrentIndex(index)
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
