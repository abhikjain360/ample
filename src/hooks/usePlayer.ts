import { useContext } from "react";
import {
    PlayerStateContext,
    PlayerProgressContext,
} from "@/context/player-context";

export function usePlayer() {
    const context = useContext(PlayerStateContext);
    if (!context) {
        throw new Error("usePlayer must be used within a PlayerProvider");
    }
    return context;
}

export function usePlayerProgress() {
    const context = useContext(PlayerProgressContext);
    if (context === undefined) {
        throw new Error(
            "usePlayerProgress must be used within a PlayerProvider",
        );
    }
    return context;
}
