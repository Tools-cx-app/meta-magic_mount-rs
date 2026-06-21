# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from asyncio import create_subprocess_exec
from asyncio.subprocess import PIPE

from . import logger


async def get_git_log(base: str, head: str = "HEAD") -> str | None:
    cmd = ["git", "log", "--oneline", "--no-decorate", f"{base}..{head}"]
    logger.info(f"Getting git log: {base}..{head}")

    process = await create_subprocess_exec(*cmd, stdout=PIPE, stderr=PIPE)
    stdout, stderr = await process.communicate()

    if process.returncode != 0:
        error_msg = stderr.decode().strip()
        logger.error(f"Git command failed: {error_msg}")
        return None

    output = stdout.decode("utf-8").strip()
    logger.info(f"Git log returned {len(output.splitlines())} commit(s)")
    return output
