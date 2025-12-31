export interface SongData {
    id: number;
    title: string;
    artist: string | null;
    duration: [number, number];
}

export interface PlaybackPayload {
    progress_frames: number;
    total_frames: number;
    is_finished: boolean;
}
