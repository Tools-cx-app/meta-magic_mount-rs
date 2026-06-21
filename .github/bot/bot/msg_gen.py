# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from . import logger, settings
from .config import TG_MSG_TEMPLATE_RELEASE, TG_MSG_TEMPLATE_CI
from .parsing import parse_release_body, parse_git_log
from .github import get_latest_release
from .gh_helpers import get_last_success_commit
from .history import get_git_log


async def generate_msg_release() -> str:
    logger.info("Generating Telegram release message")
    release = await get_latest_release()
    message = TG_MSG_TEMPLATE_RELEASE.format(
        name=release["name"],
        body=parse_release_body(release["body"]),
        url=release["html_url"],
    )
    logger.info("Generated Telegram release message")
    return message


async def generate_msg_ci() -> str:
    logger.info("Generating Telegram message")
    before_commit = settings.github_sha
    while True:
        base_hash = await get_last_success_commit(before_commit)
        if base_hash:
            git_log_raw = await get_git_log(base_hash)
            if not git_log_raw:
                before_commit = base_hash
                continue
            history_msg = parse_git_log(git_log_raw)
        else:
            logger.warning("No last success CI commit found, cannot generate message")
            history_msg = "No commit found"
        break
    commit_url = f"https://github.com/{settings.github_repository}/compare/{base_hash}...{settings.github_sha}"
    message = TG_MSG_TEMPLATE_CI.format(
        commit_message=history_msg.strip(),
        commit_url=commit_url,
        run_no=settings.run_no,
        run_id=settings.run_id,
        github_repository=settings.github_repository,
    )
    logger.info(f"Generated Telegram message: {message}")
    return message
