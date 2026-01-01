import { useCallback, useEffect, useState, useMemo } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Card, CardHeader, CardTitle } from "@/components/ui/card";
import { Folder, Plus, Music } from "lucide-react";
import { error } from "@tauri-apps/plugin-log";
import { toast } from "sonner";
import { useLocation } from "wouter";
import Loading from "@/components/Loading";
import { isError } from "@/utils";
import { useVim, useVimNavigation } from "@/hooks/useVim";

export default function Welcome() {
    const [libraries, setLibraries] = useState<string[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [selectedIndex, setSelectedIndex] = useState(0);
    const [, setLocation] = useLocation();

    useEffect(() => {
        invoke<string[]>("settings_list_libraries")
            .then(setLibraries)
            .catch((e) =>
                error(`failed to fetch libraries: ${JSON.stringify(e)}`),
            );
    }, [setLibraries]);

    const handleNewLibrary = useCallback(async () => {
        setIsLoading(true);
        try {
            const selected = await open({
                directory: true,
                multiple: false,
            });

            if (selected && typeof selected === "string") {
                await invoke<void>("library_open", { path: selected });
                toast.success("Library loaded successfully");
                setLocation("/home");
            } else {
                // User cancelled or no folder selected
                setIsLoading(false);
            }
        } catch (e) {
            console.error(e);
            toast.error("Failed to load library", {
                description: String(e),
            });
            setIsLoading(false);
        }
    }, [setIsLoading, setLocation]);

    const handleOpenLibrary = useCallback(
        async (path: string) => {
            setIsLoading(true);
            try {
                await invoke<void>("library_open", { path });
                toast.success("Library loaded successfully");
                setLocation("/home");
            } catch (e: unknown) {
                console.error(e);
                if (isError(e)) {
                    toast.error("Failed to load library", {
                        description: e.message,
                    });
                }
                setIsLoading(false);
            }
        },
        [setIsLoading, setLocation],
    );

    const handleRemoveLibrary = useCallback(
        async (index: number) => {
            const lib = libraries[index];
            if (!lib) return;

            try {
                await invoke<void>("settings_remove_library", { path: lib });
                setLibraries((prev) => prev.filter((_, i) => i !== index));
                // Adjust selection if needed
                setSelectedIndex((prev) =>
                    prev >= libraries.length - 1
                        ? Math.max(0, libraries.length - 2)
                        : prev,
                );
                toast.success("Library removed");
            } catch (e) {
                console.error(e);
                toast.error("Failed to remove library", {
                    description: String(e),
                });
            }
        },
        [libraries],
    );

    const nav = useVimNavigation(libraries, {
        onSelect: (lib) => handleOpenLibrary(lib),
    });

    // Sync nav index with state for rendering
    const updateSelection = useCallback(
        (newIndex: number) => {
            setSelectedIndex(newIndex);
        },
        [setSelectedIndex],
    );

    const bindings = useMemo(
        () => [
            {
                keys: "a",
                action: () => handleNewLibrary(),
            },
            {
                keys: "j",
                action: () => {
                    const idx = nav.next();
                    updateSelection(idx);
                },
                when: () => libraries.length > 0,
            },
            {
                keys: "k",
                action: () => {
                    const idx = nav.prev();
                    updateSelection(idx);
                },
                when: () => libraries.length > 0,
            },
            {
                keys: "gg",
                action: () => {
                    const idx = nav.first();
                    updateSelection(idx);
                },
                when: () => libraries.length > 0,
            },
            {
                keys: "G",
                action: () => {
                    const idx = nav.last();
                    updateSelection(idx);
                },
                when: () => libraries.length > 0,
            },
            {
                keys: "Enter",
                action: () => nav.select(),
                when: () => libraries.length > 0,
            },
            {
                keys: "x",
                action: () => handleRemoveLibrary(nav.getIndex()),
                when: () => libraries.length > 0,
            },
        ],
        [
            handleNewLibrary,
            handleRemoveLibrary,
            libraries.length,
            nav,
            updateSelection,
        ],
    );

    useVim({ bindings });

    if (isLoading) {
        return (
            <div className="min-h-screen flex items-center justify-center bg-background text-foreground">
                <Loading />
            </div>
        );
    }

    return (
        <div className="min-h-screen flex flex-col items-center pt-24 bg-background text-foreground selection:bg-primary selection:text-primary-foreground">
            {/* Centered Heading */}
            <div className="flex flex-col items-center space-y-2 mb-12">
                <h1 className="text-4xl font-bold tracking-tight">ample</h1>
                <p className="text-muted-foreground text-sm">
                    vim-inspired music player
                </p>
            </div>

            <div className="w-full max-w-md px-6 space-y-6">
                {/* Add Library Button */}
                <Button
                    variant="outline"
                    className="w-full h-12 border-dashed border-2 hover:border-primary hover:bg-accent/50 group transition-all cursor-pointer"
                    onClick={handleNewLibrary}
                >
                    <Plus className="mr-2 h-4 w-4 group-hover:scale-110 transition-transform" />
                    Add Library
                </Button>

                {/* Library List */}
                <div className="space-y-3">
                    <div className="flex items-center space-x-2 text-xs font-medium text-muted-foreground uppercase tracking-wider pl-1 cursor-default">
                        <Music className="h-3 w-3" />
                        <span>Libraries</span>
                    </div>

                    {libraries.length === 0 ? (
                        <div className="text-center py-8 text-muted-foreground text-sm">
                            No libraries added yet.
                        </div>
                    ) : (
                        <div className="grid gap-3">
                            {libraries.map((lib, index) => (
                                <Card
                                    key={index}
                                    className={`group hover:bg-accent/50 hover:border-primary transition-colors cursor-pointer bg-card/50 py-0 ${
                                        selectedIndex === index
                                            ? "border-primary bg-accent/50"
                                            : ""
                                    }`}
                                    onClick={() => handleOpenLibrary(lib)}
                                >
                                    <CardHeader className="p-3 flex flex-row items-center space-y-0">
                                        <Folder
                                            className={`h-4 w-4 mr-3 transition-colors ${
                                                selectedIndex === index
                                                    ? "text-primary"
                                                    : "text-muted-foreground group-hover:text-primary"
                                            }`}
                                        />
                                        <CardTitle className="text-sm font-normal truncate">
                                            {lib}
                                        </CardTitle>
                                    </CardHeader>
                                </Card>
                            ))}
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
