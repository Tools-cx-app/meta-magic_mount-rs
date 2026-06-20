# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from textwrap import shorten

from . import logger
from .config import (
    RELEASE_NOTE_MAX_LEN,
    COMMIT_LIST_MAX_LEN,
)  # COMMIT_TITLE_MAX_LEN, COMMIT_BODY_MAX_LEN


def parse_release_body(body: str) -> str:
    logger.info("Parsing release body")
    lines = list(map(str.strip, body.split("\n")))
    for line in lines:
        if line.startswith("#"):
            _, body = map(str.strip, line.split(" ", 1))
            line = f"**{body}**"
    parsed = shorten("\n".join(lines), RELEASE_NOTE_MAX_LEN, placeholder="...")
    logger.info(f"Parsed body: {parsed}")
    return parsed


# def parse_commit_message(msg: str) -> str:
#     logger.info("Parsing commit message")
#     msg = msg.replace("<", "&lt;").replace(">", "&gt;") + "\n\n"
#     title, body = msg.split("\n\n", 1)
#     title = shorten(title, COMMIT_TITLE_MAX_LEN, placeholder="...")
#     body = shorten(body, COMMIT_BODY_MAX_LEN, placeholder="...")
#     parsed = f"{title}\n\n{body}".strip()
#     logger.info(f"Parsed message: {parsed}")
#     return parsed


def parse_git_log(log: str) -> str:
    logger.info("Parsing git log")
    parsed = []
    parsed_length = 0
    for line in log.split("\n"):
        if parsed_length + len(line) + 1 <= COMMIT_LIST_MAX_LEN:
            parsed.append(line)
            parsed_length += len(line) + 1
        else:
            break
    parsed = "\n".join(parsed).replace("<", "&lt;").replace(">", "&gt;")
    logger.info(f"Parsed log: {parsed}")
    return parsed
