/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import assert from "node:assert/strict";
import test from "node:test";
import { createRealAPI } from "./api.client.ts";

const config = {
  mountsource: "KSU",
  umount: true,
  partitions: [],
  ignoreList: [],
  customMounts: [],
};

const status = {
  version: "4.0.6",
  device: { model: null },
  system: { kernel: null, selinux: null },
};

test("discovers once and sends bearer auth", async () => {
  let execCalls = 0;
  const exec = async () => {
    execCalls += 1;
    return {
      errno: 0,
      stdout: '{"port":43127,"token":"secret"}',
      stderr: "",
    };
  };
  const requests: Array<[RequestInfo | URL, RequestInit | undefined]> = [];
  const fetchMock: typeof fetch = async (input, init) => {
    requests.push([input, init]);
    return Response.json(config);
  };
  const client = createRealAPI(exec, fetchMock);

  await client.loadConfig();
  await client.loadConfig();

  assert.equal(execCalls, 1);
  assert.equal(requests[0][0], "http://127.0.0.1:43127/api/v1/config");
  assert.equal(
    (requests[0][1]?.headers as Record<string, string>).Authorization,
    "Bearer secret",
  );
});

test("rediscovers and retries exactly once after 401", async () => {
  const discoveries = ['{"port":1,"token":"old"}', '{"port":2,"token":"new"}'];
  let execCalls = 0;
  const exec = async () => ({
    errno: 0,
    stdout: discoveries[execCalls++],
    stderr: "",
  });
  const responses = [new Response("", { status: 401 }), Response.json(status)];
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => responses[fetchCalls++];

  await createRealAPI(exec, fetchMock).getVersion();

  assert.equal(execCalls, 2);
  assert.equal(fetchCalls, 2);
});

test("surfaces daemon errors without retrying", async () => {
  const exec = async () => ({
    errno: 0,
    stdout: '{"port":1,"token":"secret"}',
    stderr: "",
  });
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => {
    fetchCalls += 1;
    return new Response(
      '{"error":{"code":"invalidConfig","message":"invalid config"}}',
      { status: 400 },
    );
  };

  await assert.rejects(createRealAPI(exec, fetchMock).saveConfig(config), {
    message: "invalid config",
  });
  assert.equal(fetchCalls, 1);
});

test("does not retry a malformed daemon error response", async () => {
  const exec = async () => ({
    errno: 0,
    stdout: '{"port":1,"token":"secret"}',
    stderr: "",
  });
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => {
    fetchCalls += 1;
    return Response.json({}, { status: 400 });
  };

  await assert.rejects(createRealAPI(exec, fetchMock).saveConfig(config), {
    message: "daemon request failed (400)",
  });
  assert.equal(fetchCalls, 1);
});

test("rediscovers and retries exactly once after a network TypeError", async () => {
  let execCalls = 0;
  const exec = async () => ({
    errno: 0,
    stdout: JSON.stringify({ port: ++execCalls, token: `token-${execCalls}` }),
    stderr: "",
  });
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => {
    fetchCalls += 1;
    if (fetchCalls === 1) throw new TypeError("network failed");
    return Response.json(config);
  };

  await createRealAPI(exec, fetchMock).loadConfig();

  assert.equal(execCalls, 2);
  assert.equal(fetchCalls, 2);
});

test("does not cache a failed discovery", async () => {
  let execCalls = 0;
  const exec = async () => {
    execCalls += 1;
    if (execCalls === 1) {
      return { errno: 1, stdout: "", stderr: "daemon is starting" };
    }
    return {
      errno: 0,
      stdout: '{"port":1,"token":"secret"}',
      stderr: "",
    };
  };
  const client = createRealAPI(exec, async () => Response.json(config));

  await assert.rejects(client.loadConfig(), /daemon is starting/);
  assert.deepEqual(await client.loadConfig(), config);
  assert.equal(execCalls, 2);
});

test("invalidates discovery without replaying a non-idempotent action", async () => {
  let execCalls = 0;
  const exec = async () => {
    execCalls += 1;
    return {
      errno: 0,
      stdout: '{"port":1,"token":"secret"}',
      stderr: "",
    };
  };
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => {
    fetchCalls += 1;
    if (fetchCalls === 1) {
      throw new TypeError("network failed");
    }
    return new Response(null, { status: 204 });
  };

  const client = createRealAPI(exec, fetchMock);
  await assert.rejects(
    client.openLink("https://example.com"),
    /network failed/,
  );
  assert.equal(execCalls, 1);
  assert.equal(fetchCalls, 1);

  await client.openLink("https://example.com");
  assert.equal(execCalls, 2);
  assert.equal(fetchCalls, 2);
});

test("shares only concurrent status requests", async () => {
  const exec = async () => ({
    errno: 0,
    stdout: '{"port":1,"token":"secret"}',
    stderr: "",
  });
  let fetchCalls = 0;
  const fetchMock: typeof fetch = async () => {
    fetchCalls += 1;
    return Response.json(status);
  };
  const client = createRealAPI(exec, fetchMock);

  assert.deepEqual(
    await Promise.all([
      client.getVersion(),
      client.getDeviceStatus(),
      client.getSystemInfo(),
    ]),
    ["4.0.6", { model: null }, { kernel: null, selinux: null }],
  );
  assert.equal(fetchCalls, 1);

  await client.getVersion();
  assert.equal(fetchCalls, 2);
});

test("uses daemon routes and JSON request bodies", async () => {
  const exec = async () => ({
    errno: 0,
    stdout: '{"port":1,"token":"secret"}',
    stderr: "",
  });
  const requests: Array<[string, RequestInit | undefined]> = [];
  const responses = [
    new Response(null, { status: 204 }),
    Response.json([]),
    new Response(null, { status: 204 }),
  ];
  const fetchMock: typeof fetch = async (input, init) => {
    requests.push([String(input), init]);
    const response = responses.shift();
    if (!response) throw new Error("unexpected request");
    return response;
  };
  const client = createRealAPI(exec, fetchMock);

  await client.saveConfig(config);
  await client.scanModules();
  await client.openLink("https://example.com/path");

  assert.deepEqual(
    requests.map(([url, init]) => [url, init?.method, init?.body]),
    [
      [
        "http://127.0.0.1:1/api/v1/actions/reload",
        "POST",
        JSON.stringify(config),
      ],
      ["http://127.0.0.1:1/api/v1/modules", undefined, undefined],
      [
        "http://127.0.0.1:1/api/v1/actions/open-link",
        "POST",
        '{"url":"https://example.com/path"}',
      ],
    ],
  );
});

test("accepts an empty successful 202 response", async () => {
  const exec = async () => ({
    errno: 0,
    stdout: '{"port":1,"token":"secret"}',
    stderr: "",
  });
  const fetchMock: typeof fetch = async () =>
    new Response(null, { status: 202 });

  await createRealAPI(exec, fetchMock).reboot();
});

test("rejects invalid discovery data before requesting", async () => {
  const invalidConnections = [
    '{"port":0,"token":"secret"}',
    '{"port":65536,"token":"secret"}',
    '{"port":1.5,"token":"secret"}',
    '{"port":1,"token":""}',
  ];

  for (const stdout of invalidConnections) {
    const exec = async () => ({ errno: 0, stdout, stderr: "" });
    await assert.rejects(createRealAPI(exec).loadConfig(), /connection info/);
  }
});
