# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

from dataclasses import dataclass
from logging import getLogger
from pydantic_settings import BaseSettings


class Settings(BaseSettings):
    bot_token: str = ""
    chat_id: int = 0
    run_no: int = 0
    run_id: int = 0
    github_repository: str = ""
    github_token: str = ""
    github_sha: str = ""
    is_release: bool = False
    github_event_path: str | None = None
    github_event_name: str | None = None
    github_ref_name: str | None = None
    github_ref: str | None = None

@dataclass
class _Cache:
    workflow_file: str | None = None


settings = Settings()   # type: ignore
cache = _Cache()
logger = getLogger(__name__)


# Global state cache


async def main():
    from .config import (
        TG_MSG_EXPECTED_PARSE_MODE_RELEASE,
        TG_MSG_EXPECTED_PARSE_MODE_CI,
    )
    from .msg_gen import generate_msg_release, generate_msg_ci
    from .util import get_dist
    from .telegram import post

    logger.info("Starting main function")
    if settings.is_release:
        msg = await generate_msg_release()
        parse_mode = TG_MSG_EXPECTED_PARSE_MODE_RELEASE
    else:
        msg = await generate_msg_ci()
        parse_mode = TG_MSG_EXPECTED_PARSE_MODE_CI
    files = get_dist()
    await post(msg, files, parse_mode)
    logger.info("Post done successfully")
