import { Play, Clock, PlayCircle } from "lucide-react";
import { TableVirtuoso, VirtuosoHandle } from "react-virtuoso";
import { SongData } from "@/types";
import { formatDuration } from "@/lib/utils";
import { memo } from "react";

interface SongListProps {
    songs: SongData[];
    currentSong: SongData | null;
    selectedIndex: number;
    onPlay: (song: SongData) => void;
    onSelect?: (index: number) => void;
    virtuosoRef: React.RefObject<VirtuosoHandle | null>;
    overscan?: number;
}

function SongList({
    songs,
    currentSong,
    selectedIndex,
    onPlay,
    onSelect,
    virtuosoRef,
    overscan = 200,
}: SongListProps) {
    return (
        <TableVirtuoso
            ref={virtuosoRef}
            data={songs}
            overscan={overscan}
            className="h-full w-full scroll-pt-11"
            fixedHeaderContent={() => (
                <tr className="bg-muted/70 backdrop-blur-md border-b-2 border-border text-xs text-muted-foreground uppercase">
                    <th className="px-4 py-3 font-medium w-12 text-center bg-muted/70 backdrop-blur-md">
                        #
                    </th>
                    <th className="px-4 py-3 font-medium bg-muted/70 backdrop-blur-md text-left">
                        Title
                    </th>
                    <th className="px-4 py-3 font-medium bg-muted/70 backdrop-blur-md text-left hidden sm:table-cell">
                        Artist
                    </th>
                    <th className="px-4 py-3 font-medium text-right w-24 bg-muted/70 backdrop-blur-md hidden sm:table-cell">
                        <Clock className="h-3 w-3 inline-block" />
                    </th>
                </tr>
            )}
            itemContent={(index, song) => {
                const isPlayingCurrent = currentSong?.id === song.id;

                const textColorClass = isPlayingCurrent
                    ? "text-chart-4"
                    : "text-foreground";
                const mutedTextClass = isPlayingCurrent
                    ? "text-chart-4/70"
                    : "text-muted-foreground";

                return (
                    <>
                        <td
                            className={`px-4 py-3 text-center font-mono text-xs w-12 ${textColorClass}`}
                        >
                            {isPlayingCurrent ? (
                                <Play className="h-3 w-3 mx-auto fill-current" />
                            ) : (
                                <span className="opacity-50">{index + 1}</span>
                            )}
                        </td>
                        <td
                            className={`px-4 py-3 font-medium truncate max-w-[50vw] sm:max-w-[30vw] ${textColorClass}`}
                        >
                            <div className="flex items-center gap-2">
                                <span className="truncate">{song.title}</span>
                                <button
                                    onClick={(e) => {
                                        e.stopPropagation();
                                        onPlay(song);
                                    }}
                                    className="opacity-0 group-hover:opacity-100 transition-opacity flex-shrink-0 text-primary hover:text-primary/80"
                                    title="Play"
                                >
                                    <PlayCircle className="h-4 w-4" />
                                </button>
                            </div>
                        </td>
                        <td
                            className={`px-4 py-3 truncate max-w-[20vw] hidden sm:table-cell ${mutedTextClass}`}
                        >
                            {song.artist || "Unknown Artist"}
                        </td>
                        <td
                            className={`px-4 py-3 text-right font-mono text-xs w-24 hidden sm:table-cell ${mutedTextClass}`}
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
                        "group cursor-pointer select-none border-l-2 transition-colors active:bg-accent/50 ";

                    if (isPlayingCurrent) {
                        className += "border-l-chart-4 ";
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
                            onClick={() => onSelect?.(index)}
                            onDoubleClick={() => onPlay(songs[index])}
                        />
                    );
                },
            }}
        />
    );
}

export default memo(SongList);
