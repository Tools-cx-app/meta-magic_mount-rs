# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

import html
from . import logger
from .config import PARSING_MAX_LEN


def parse_git_log(log: str) -> str:
    logger.info("Parsing git log")
    lines = [line for line in log.split("\n") if line.strip()]
    if not lines:
        return "No commit found"
    parsed = []
    parsed_length = 0
    for line in lines:
        parsed_length += len(line) + 1
        if parsed_length <= PARSING_MAX_LEN:
            parsed.append(line)
        else:
            break
    parsed_str = html.escape("\n".join(reversed(parsed)), quote=False)
    if len(lines) > len(parsed):
        parsed_str = f"...{len(lines)-len(parsed)} more commits...\n" + parsed_str
    logger.info(f"Parsed log: {parsed_str}")
    return parsed_str
