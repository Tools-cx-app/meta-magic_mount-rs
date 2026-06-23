/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { PATHS } from "./constants.ts";
import type {
  AppAPI,
  AppConfig,
  ConnectionInfo,
  ErrorResponse,
  Status,
} from "./types.ts";

interface KsuExecResult {
  errno: number;
  stdout: string;
  stderr: string;
}

export type KsuExec = (cmd: string) => Promise<KsuExecResult>;

export function createRealAPI(
  exec: KsuExec,
  request: typeof fetch = fetch,
): AppAPI {
  let connectionPromise: Promise<ConnectionInfo> | null = null;
  let statusPromise: Promise<Status> | null = null;

  async function discover(): Promise<ConnectionInfo> {
    connectionPromise ??= exec(`cat ${PATHS.CONNECTION}`).then((result) => {
      if (result.errno !== 0) {
        throw new Error(result.stderr || "failed to discover daemon");
      }

      let connection: ConnectionInfo;
      try {
        connection = JSON.parse(result.stdout) as ConnectionInfo;
      } catch {
        throw new Error("invalid daemon connection info");
      }
      if (
        !Number.isInteger(connection.port) ||
        connection.port < 1 ||
        connection.port > 65535 ||
        typeof connection.token !== "string" ||
        connection.token.length === 0
      ) {
        throw new Error("invalid daemon connection info");
      }
      return connection;
    });
    try {
      return await connectionPromise;
    } catch (error) {
      connectionPromise = null;
      throw error;
    }
  }

  async function requestJSON<T>(
    path: string,
    init: RequestInit = {},
  ): Promise<T> {
    const method = (init.method ?? "GET").toUpperCase();
    const canRetryNetworkFailure =
      method === "GET" || method === "HEAD" || method === "PUT";
    for (let attempt = 0; attempt < 2; attempt += 1) {
      const connection = await discover();
      let response: Response;
      try {
        response = await request(
          `http://127.0.0.1:${connection.port}/api/v1${path}`,
          {
            ...init,
            headers: {
              "Content-Type": "application/json",
              ...init.headers,
              Authorization: `Bearer ${connection.token}`,
            },
          },
        );
      } catch (error) {
        if (error instanceof TypeError) {
          connectionPromise = null;
          if (attempt === 0 && canRetryNetworkFailure) {
            continue;
          }
        }
        throw error;
      }
      if (response.status === 401 && attempt === 0) {
        connectionPromise = null;
        continue;
      }
      if (!response.ok) {
        const payload = (await response
          .json()
          .catch(() => null)) as ErrorResponse | null;
        throw new Error(
          payload?.error?.message ??
            `daemon request failed (${response.status})`,
        );
      }

      const body = await response.text();
      return (body ? JSON.parse(body) : undefined) as T;
    }
    throw new Error("daemon request failed after reconnect");
  }

  function getStatus(): Promise<Status> {
    if (!statusPromise) {
      statusPromise = requestJSON<Status>("/status").finally(() => {
        statusPromise = null;
      });
    }
    return statusPromise;
  }

  return {
    loadConfig: () => requestJSON<AppConfig>("/config"),
    saveConfig: (config) =>
      requestJSON<void>("/config", {
        method: "PUT",
        body: JSON.stringify(config),
      }),
    scanModules: () => requestJSON("/modules"),
    getSystemInfo: async () => (await getStatus()).system,
    getDeviceStatus: async () => (await getStatus()).device,
    getVersion: async () => (await getStatus()).version,
    openLink: (url) =>
      requestJSON<void>("/actions/open-link", {
        method: "POST",
        body: JSON.stringify({ url }),
      }),
    reboot: () =>
      requestJSON<void>("/actions/reboot", {
        method: "POST",
      }),
  };
}
