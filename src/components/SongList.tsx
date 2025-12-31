import { Play, Clock } from "lucide-react";
import { TableVirtuoso, VirtuosoHandle } from "react-virtuoso";
import { SongData } from "@/types";

interface SongListProps {
    songs: SongData[];
    currentSong: SongData | null;
    selectedIndex: number;
    onSelect: (index: number) => void;
    onPlay: (song: SongData) => void;
    virtuosoRef: React.RefObject<VirtuosoHandle | null>;
    overscan?: number;
}

export default function SongList({
    songs,
    currentSong,
    selectedIndex,
    onSelect,
    onPlay,
    virtuosoRef,
    overscan = 200,
}: SongListProps) {
    const formatDuration = (duration: [number, number]) => {
        const [minutes, seconds] = duration;
        return `${minutes}:${seconds.toString().padStart(2, "0")}`;
    };

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
                    <th className="px-4 py-3 font-medium bg-muted/70 backdrop-blur-md text-left">
                        Artist
                    </th>
                    <th className="px-4 py-3 font-medium text-right w-24 bg-muted/70 backdrop-blur-md">
                        <Clock className="h-3 w-3 inline-block" />
                    </th>
                </tr>
            )}
            itemContent={(index, song) => {
                const isPlayingCurrent = currentSong?.id === song.id;

                // Text color logic
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
                        className += "border-l-chart-4 ";
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
                            onClick={() => onSelect(index)}
                            onMouseEnter={() => onSelect(index)}
                            onDoubleClick={() => onPlay(songs[index])}
                        />
                    );
                },
            }}
        />
    );
}
