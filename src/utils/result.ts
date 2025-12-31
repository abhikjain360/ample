export type ErrorKind = "io" | "settings";

export interface TauriError {
    kind: ErrorKind;
    message: string;
}

export type Result<T> = T | TauriError;

export function isError<T>(result: Result<T>): result is TauriError {
    return (
        typeof result === "object" &&
        result !== null &&
        "kind" in result &&
        "message" in result
    );
}

export function isOk<T>(result: Result<T>): result is T {
    return !isError(result);
}
