export interface IpcErrorPayload {
  readonly kind: string;
  readonly message: string;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function isIpcErrorPayload(error: unknown): error is IpcErrorPayload {
  return isRecord(error) && typeof error.kind === "string" && typeof error.message === "string";
}

export function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (isRecord(error) && typeof error.message === "string") return error.message;
  return String(error);
}

export function errorKind(error: unknown): string | undefined {
  if (!isRecord(error)) return undefined;
  return typeof error.kind === "string" ? error.kind : undefined;
}

export function normalizeError(error: unknown): Error {
  if (error instanceof Error) return error;

  const normalized = new Error(errorMessage(error));
  const kind = errorKind(error);
  return kind === undefined ? normalized : Object.assign(normalized, { kind });
}
