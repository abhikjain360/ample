import "./App.css";

import { useEffect, useMemo } from "react";
import { Route, Switch, useLocation } from "wouter";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import Welcome from "./pages/Welcome";
import Home from "./pages/Home";
import Queue from "./pages/Queue";
import { Toaster } from "@/components/ui/sonner";
import { PlayerProvider, usePlayer } from "@/context/PlayerContext";
import { Player } from "@/components/Player";
import { useVim } from "@/hooks/useVim";

function AppContent() {
    const { playNext, playPrev, clearQueue } = usePlayer();
    const [location, setLocation] = useLocation();

    useEffect(() => {
        const unlisten = getCurrentWindow().onCloseRequested(async () => {
            await invoke<void>("settings_save");
        });

        return () => {
            unlisten.then((fn) => fn());
        };
    }, []);

    const bindings = useMemo(
        () => [
            {
                keys: "q",
                action: () => {
                    if (location === "/home") setLocation("/queue");
                    else if (location === "/queue") setLocation("/home");
                },
                noRepeat: true,
            },
            {
                keys: "n",
                action: () => playNext(),
                noRepeat: true,
            },
            {
                keys: "p",
                action: () => playPrev(),
                noRepeat: true,
            },
            {
                keys: "X",
                action: () => clearQueue(),
                noRepeat: true,
            },
        ],
        [location, setLocation, playNext, playPrev, clearQueue],
    );

    useVim({ bindings });

    return (
        <div className="h-screen w-full flex flex-col overflow-hidden bg-background text-foreground">
            <div className="flex-1 overflow-hidden relative">
                <Switch>
                    <Route path="/" component={Welcome} />
                    <Route path="/home" component={Home} />
                    <Route path="/queue" component={Queue} />
                    <Route>404: Page Not Found</Route>
                </Switch>
            </div>
            <Player />
            <Toaster />
        </div>
    );
}

function App() {
    return (
        <PlayerProvider>
            <AppContent />
        </PlayerProvider>
    );
}

export default App;
