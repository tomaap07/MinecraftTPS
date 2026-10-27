export const CONSOLE_LINE_LIMIT = 1500;
export const CONSOLE_IDLE_MS = 12 * 60 * 1000;


export function consoleCanExpire(now: number, lastActivity: number, gameActive: boolean): boolean {
  return !gameActive && now >= lastActivity && now - lastActivity >= CONSOLE_IDLE_MS;
}

export function boundedConsole<T>(lines: readonly T[]): T[] {
  return lines.slice(-CONSOLE_LINE_LIMIT);
}
