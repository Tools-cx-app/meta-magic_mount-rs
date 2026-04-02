import type { MountMode } from "../types";

export function normalizeModuleMode(
  mode: string | null | undefined,
): MountMode | string {
  if (!mode) {
    return "magic";
  }

  if (mode === "auto") {
    return "overlay";
  }

  if (mode === "umount" || mode === "disabled") {
    return "ignore";
  }

  return mode;
}
