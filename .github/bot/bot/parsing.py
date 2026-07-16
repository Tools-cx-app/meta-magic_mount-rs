# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

from . import logger
from .config import PARSING_MAX_LEN


def parse_git_log(log: str) -> str:
    logger.info("Parsing git log")
    lines = log.split("\n")
    parsed = []
    parsed_length = 0
    for line in lines:
        parsed_length += len(line) + 1
        if parsed_length <= PARSING_MAX_LEN:
            parsed.append(line)
        else:
            break
    parsed_str = "\n".join(reversed(parsed)).replace("<", "&lt;").replace(">", "&gt;")
    if len(lines) > len(parsed):
        parsed_str = f"...{len(lines)-len(parsed)} more commits...\n" + parsed_str
    logger.info(f"Parsed log: {parsed_str}")
    return parsed_str
