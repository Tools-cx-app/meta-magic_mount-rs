# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

from . import logger, settings
from .config import TG_MSG_TEMPLATE_RELEASE, TG_MSG_TEMPLATE_CI
from .parsing import parse_git_log
from .github import get_latest_release, get_commit
from .gh_helpers import (
    get_last_success_commit,
    get_git_log,
    load_event_payload,
    get_commits_from_event,
    get_url_from_event,
)


async def generate_msg_release() -> str:
    logger.info("Generating Telegram release message")
    release = await get_latest_release()
    message = TG_MSG_TEMPLATE_RELEASE.format(
        name=release["name"],
        url=release["html_url"],
    )
    logger.info("Generated Telegram release message")
    return message


async def get_single_commit_fallback() -> tuple[str, str]:
    commit_url = f"https://github.com/{settings.github_repository}/commit/{settings.github_sha}"
    try:
        data = await get_commit(settings.github_sha)
        sha = data.get("sha", settings.github_sha)
        message = data.get("commit", {}).get("message", "")
        first_line = message.splitlines()[0] if message else "No commit message"
        msg = f"{sha[:7]} {first_line}"
        return parse_git_log(msg), commit_url
    except Exception as e:
        logger.warning(f"Failed to fetch single commit {settings.github_sha}: {e}")
        return f"{settings.github_sha[:7]} (commit details unavailable)", commit_url


async def generate_msg_ci() -> str:
    logger.info("Generating Telegram message")
    branch = settings.github_ref_name

    history_msg: str | None = None
    commit_url: str | None = None

    # 1. Primary strategy: Find last successful CI run on this branch
    base_hash = await get_last_success_commit(
        before_commit=settings.github_sha, branch=branch
    )
    if base_hash and base_hash != settings.github_sha:
        git_log_raw = await get_git_log(base_hash, settings.github_sha)
        if git_log_raw:
            history_msg = parse_git_log(git_log_raw)
            commit_url = (
                f"https://github.com/{settings.github_repository}/compare/"
                f"{base_hash}...{settings.github_sha}"
            )
            logger.info("Successfully generated message from last successful CI run comparison")

    # 2. Backup strategy: Use GITHUB_EVENT_PATH / event payload
    if not history_msg or not commit_url:
        logger.info("Primary strategy did not yield commits; using event payload backup strategy")
        payload = load_event_payload()

        # 2a. Try compare using payload's `before` commit
        before_sha = payload.get("before")
        if (
            before_sha
            and set(before_sha) != {"0"}
            and before_sha != settings.github_sha
        ):
            git_log_raw = await get_git_log(before_sha, settings.github_sha)
            if git_log_raw:
                history_msg = parse_git_log(git_log_raw)
                compare_url = payload.get("compare")
                commit_url = compare_url or (
                    f"https://github.com/{settings.github_repository}/compare/"
                    f"{before_sha}...{settings.github_sha}"
                )
                logger.info("Successfully generated message using event payload before-commit comparison")

        # 2b. Extract commits directly from event payload
        if not history_msg or not commit_url:
            event_commits = get_commits_from_event(payload)
            if event_commits:
                history_msg = parse_git_log("\n".join(event_commits))
                commit_url = get_url_from_event(payload)
                logger.info(f"Successfully generated message from {len(event_commits)} commits in event payload")

    # 3. Final fallback: single commit details
    if not history_msg or not commit_url:
        logger.warning("No commits found via primary or event strategies; falling back to single commit info")
        history_msg, commit_url = await get_single_commit_fallback()

    message = TG_MSG_TEMPLATE_CI.format(
        commit_message=history_msg.strip(),
        commit_url=commit_url,
        run_no=settings.run_no,
        run_id=settings.run_id,
        github_repository=settings.github_repository,
    )
    logger.info(f"Generated Telegram message: {message}")
    return message
