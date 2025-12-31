import { useEffect, useRef, useCallback, useMemo } from "react";

export type KeyBinding = {
    keys: string | string[];
    action: (context: { repeat: boolean }) => void;
    when?: () => boolean;
    /** If true, action runs on initial press only, ignoring key repeat */
    noRepeat?: boolean;
};

export type VimConfig = {
    bindings: KeyBinding[];
    sequenceTimeout?: number;
};

const DEFAULT_SEQUENCE_TIMEOUT = 500;

export function useVim(config: VimConfig) {
    const sequenceRef = useRef<string>("");
    const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
    const bindingsRef = useRef(config.bindings);
    const processingRef = useRef(false);

    // Update bindings ref when config changes
    bindingsRef.current = config.bindings;

    const resetSequence = useCallback(() => {
        sequenceRef.current = "";
        if (timeoutRef.current) {
            clearTimeout(timeoutRef.current);
            timeoutRef.current = null;
        }
    }, []);

    const handleKeyDown = useCallback(
        (e: KeyboardEvent) => {
            // Ignore if typing in an input
            const target = e.target as HTMLElement;
            if (
                target.tagName === "INPUT" ||
                target.tagName === "TEXTAREA" ||
                target.isContentEditable
            ) {
                return;
            }

            // Ignore modifier-only keys
            if (["Control", "Alt", "Shift", "Meta"].includes(e.key)) {
                return;
            }

            // Throttle repeat events to prevent event flooding (limit to ~60fps)
            if (e.repeat && processingRef.current) {
                e.preventDefault();
                return;
            }

            // Skip if still processing previous action
            if (processingRef.current) {
                e.preventDefault();
                return;
            }

            // Build key representation
            let key = e.key;
            if (e.ctrlKey) key = `C-${key}`;
            if (e.altKey) key = `A-${key}`;
            if (e.metaKey) key = `M-${key}`;

            // Reset timeout
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }

            // Add to sequence
            sequenceRef.current += key;
            const currentSequence = sequenceRef.current;
            const bindings = bindingsRef.current;

            // Helper to execute binding
            const executeBinding = (binding: KeyBinding, count: number = 1) => {
                // Check condition if exists
                if (binding.when && !binding.when()) {
                    return false;
                }

                // Handle noRepeat bindings
                if (binding.noRepeat && e.repeat) {
                    e.preventDefault();
                    resetSequence();
                    return true;
                }

                e.preventDefault();

                // Use requestAnimationFrame to batch with next paint
                processingRef.current = true;

                requestAnimationFrame(() => {
                    // Execute action 'count' times
                    for (let i = 0; i < count; i++) {
                        binding.action({ repeat: e.repeat });
                    }
                    processingRef.current = false;
                });

                resetSequence();
                return true;
            };

            // 1. Check for exact full match
            for (const binding of bindings) {
                const keys = Array.isArray(binding.keys)
                    ? binding.keys
                    : [binding.keys];
                for (const bindingKey of keys) {
                    if (bindingKey === currentSequence) {
                        if (executeBinding(binding)) return;
                        // If condition failed, continue searching
                    }
                }
            }

            // 2. Check for numeric prefix match
            // Pattern: [count][command]
            // Only look for count if the full sequence wasn't an exact match (or condition failed)
            const countMatch = currentSequence.match(/^(\d+)(.+)$/);
            if (countMatch) {
                const count = parseInt(countMatch[1], 10);
                const commandPart = countMatch[2];

                // If sequence starts with 0, and 0 is not a valid count (it's a movement),
                // we should have caught it in exact match if "0" was bound.
                // But standard Vim: 0 is start of line, 1-9 starts count.
                // Here we just parse any number.

                for (const binding of bindings) {
                    const keys = Array.isArray(binding.keys)
                        ? binding.keys
                        : [binding.keys];
                    for (const bindingKey of keys) {
                        if (bindingKey === commandPart) {
                            if (executeBinding(binding, count)) return;
                        }
                    }
                }
            }

            // 3. Check if current sequence is a prefix of any binding (or [count]prefix)
            let isPrefix = false;

            // Check if full sequence is prefix of any binding
            for (const binding of bindings) {
                const keys = Array.isArray(binding.keys)
                    ? binding.keys
                    : [binding.keys];
                for (const bindingKey of keys) {
                    if (
                        bindingKey.startsWith(currentSequence) &&
                        bindingKey !== currentSequence
                    ) {
                        isPrefix = true;
                        break;
                    }
                }
                if (isPrefix) break;
            }

            // If not a direct prefix, check if it's a number followed by a prefix
            if (!isPrefix) {
                // If sequence is just digits, it's a prefix for any binding (potentially)
                if (/^\d+$/.test(currentSequence)) {
                    // Wait, if it's just "s", /^\d+$/ is false.
                    // But if sequence is "1", it is true.
                    isPrefix = true;
                } else {
                    // Check if it's [count][partial_command]
                    const prefixMatch = currentSequence.match(/^(\d+)(.+)$/);
                    if (prefixMatch) {
                        const commandPart = prefixMatch[2];
                        for (const binding of bindings) {
                            const keys = Array.isArray(binding.keys)
                                ? binding.keys
                                : [binding.keys];
                            for (const bindingKey of keys) {
                                if (
                                    bindingKey.startsWith(commandPart) &&
                                    bindingKey !== commandPart
                                ) {
                                    isPrefix = true;
                                    break;
                                }
                            }
                            if (isPrefix) break;
                        }
                    }
                }
            }

            if (isPrefix) {
                // Set timeout to reset sequence
                timeoutRef.current = setTimeout(() => {
                    resetSequence();
                }, config.sequenceTimeout ?? DEFAULT_SEQUENCE_TIMEOUT);
            } else {
                // No match and not a prefix, reset
                resetSequence();
            }
        },
        [config.sequenceTimeout, resetSequence],
    );

    useEffect(() => {
        window.addEventListener("keydown", handleKeyDown);
        return () => {
            window.removeEventListener("keydown", handleKeyDown);
            if (timeoutRef.current) {
                clearTimeout(timeoutRef.current);
            }
        };
    }, [handleKeyDown]);

    return { resetSequence };
}

// Navigation helper for list-based views
export function useVimNavigation<T>(
    items: T[],
    options?: {
        loop?: boolean;
        onSelect?: (item: T, index: number) => void;
    },
) {
    const indexRef = useRef(0);
    const itemsRef = useRef(items);
    itemsRef.current = items;

    const getIndex = useCallback(() => indexRef.current, []);

    const setIndex = useCallback(
        (index: number) => {
            const len = itemsRef.current.length;
            if (len === 0) return;

            if (options?.loop) {
                indexRef.current = ((index % len) + len) % len;
            } else {
                indexRef.current = Math.max(0, Math.min(len - 1, index));
            }
        },
        [options?.loop],
    );

    const next = useCallback(() => {
        setIndex(indexRef.current + 1);
        return indexRef.current;
    }, [setIndex]);

    const prev = useCallback(() => {
        setIndex(indexRef.current - 1);
        return indexRef.current;
    }, [setIndex]);

    const first = useCallback(() => {
        setIndex(0);
        return indexRef.current;
    }, [setIndex]);

    const last = useCallback(() => {
        setIndex(itemsRef.current.length - 1);
        return indexRef.current;
    }, [setIndex]);

    const select = useCallback(() => {
        const idx = indexRef.current;
        const item = itemsRef.current[idx];
        if (item && options?.onSelect) {
            options.onSelect(item, idx);
        }
    }, [options]);

    return useMemo(
        () => ({
            getIndex,
            setIndex,
            next,
            prev,
            first,
            last,
            select,
        }),
        [getIndex, setIndex, next, prev, first, last, select],
    );
}
