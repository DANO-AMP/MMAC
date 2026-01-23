/**
 * Format a byte size into a human-readable string.
 */
export function formatSize(bytes: number): string {
  if (!bytes || bytes <= 0 || !isFinite(bytes)) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(
    Math.floor(Math.log(bytes) / Math.log(k)),
    sizes.length - 1
  );
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

/**
 * Format a speed in bytes per second into a human-readable string.
 */
export function formatSpeed(bytesPerSec: number): string {
  if (!bytesPerSec || bytesPerSec <= 0 || !isFinite(bytesPerSec)) return "0 B/s";
  const k = 1024;
  const sizes = ["B/s", "KB/s", "MB/s", "GB/s"];
  const i = Math.min(
    Math.floor(Math.log(bytesPerSec) / Math.log(k)),
    sizes.length - 1
  );
  return parseFloat((bytesPerSec / Math.pow(k, i)).toFixed(1)) + " " + sizes[i];
}

/**
 * Format a timestamp into a localized date string.
 */
export function formatDate(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleDateString();
}

/**
 * Format a timestamp into a localized time string.
 */
export function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString();
}

/**
 * Format a timestamp into a localized date and time string.
 */
export function formatDateTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleString();
}

/**
 * Format a percentage value.
 */
export function formatPercentage(value: number, decimals: number = 1): string {
  if (!isFinite(value)) return "0%";
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format a temperature in Celsius.
 */
export function formatTemperature(celsius: number): string {
  if (!isFinite(celsius)) return "N/A";
  return `${celsius.toFixed(1)}°C`;
}
