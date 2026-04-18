/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type { AppAPI, AppConfig } from "../types";
import { DEFAULT_CONFIG } from "./constants";

const MOCK_DELAY = 600;

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export const MockAPI: AppAPI = {
  loadConfig: async () => {
    await delay(MOCK_DELAY);
    console.log("[MockAPI] loadConfig");

    return {
      ...DEFAULT_CONFIG,
      mountsource: "KSU",
      umount: true,
      partitions: ["product", "system_ext", "vendor"],
    };
  },

  saveConfig: async (config: AppConfig) => {
    await delay(MOCK_DELAY);
    console.log("[MockAPI] saveConfig:", config);
  },

  scanModules: async () => {
    await delay(MOCK_DELAY);
    console.log("[MockAPI] scanModules");

    return [
      {
        id: "youtube-revanced",
        name: "YouTube ReVanced",
        version: "18.20.39",
        author: "ReVanced Team",
        description: "YouTube ReVanced Module",
        is_mounted: true,
      },
      {
        id: "pixelfy-gphotos",
        name: "Pixelfy GPhotos",
        version: "2.1",
        author: "PixelProps",
        description: "Unlimited Google Photos backup for Pixel devices.",
        is_mounted: true,
      },
      {
        id: "sound-enhancer",
        name: "Sound Enhancer",
        version: "1.0",
        author: "AudioMod",
        description: "Improves system audio quality. Currently disabled.",
        is_mounted: false,
      },
    ];
  },

  getSystemInfo: async () => {
    await delay(MOCK_DELAY);

    return {
      kernel: "5.10.101-android12-9-00001-g532145",
      selinux: "Enforcing",
    };
  },

  getDeviceStatus: async () => {
    await delay(MOCK_DELAY);

    return {
      model: "Pixel 8 Pro (Mock)",
    };
  },

  getVersion: async () => {
    await delay(MOCK_DELAY);

    return "1.2.0-mock";
  },

  reboot: async () => {
    console.log("[MockAPI] Reboot requested");
    // eslint-disable-next-line no-alert
    alert("Reboot requested (Mock)");
  },

  openLink: async (url: string) => {
    console.log("[MockAPI] Open link:", url);
    window.open(url, "_blank");
  },
};
