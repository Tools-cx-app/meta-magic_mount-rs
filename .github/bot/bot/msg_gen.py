# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

from . import logger, settings
from .config import TG_MSG_TEMPLATE_RELEASE, TG_MSG_TEMPLATE_CI
from .parsing import parse_git_log
from .gh_helpers import (
    get_git_log,
    load_event_payload,
    get_commits_from_event,
    get_url_from_event,
    run_git,
)


def _release_tag(payload: dict) -> str:
    tag = settings.github_ref_name or ""
    github_ref = settings.github_ref or ""
    if not tag and github_ref.startswith("refs/tags/"):
        tag = github_ref.removeprefix("refs/tags/")
    if not tag:
        object_name = payload.get("ref", "")
        if object_name.startswith("refs/tags/"):
            tag = object_name.removeprefix("refs/tags/")
    return tag.strip()


async def generate_msg_release() -> str:
    logger.info("Generating Telegram release message")
    payload = load_event_payload()
    repository = payload.get("repository", {}).get("full_name") or settings.github_repository
    tag = _release_tag(payload)
    if not tag:
        tag = (await run_git("describe", "--tags", "--exact-match", "HEAD")).strip()
    if not tag:
        tag = (await run_git("describe", "--tags", "--abbrev=0")).strip()

    name = tag or "release"
    if tag:
        url = f"https://github.com/{repository}/releases/tag/{tag}"
    else:
        url = f"https://github.com/{repository}/actions/runs/{settings.run_id}"
    message = TG_MSG_TEMPLATE_RELEASE.format(name=name, url=url)
    logger.info("Generated Telegram release message")
    return message


def _git_commit_record(message: str) -> str:
    sha = settings.github_sha[:7]
    first_line = message.splitlines()[0] if message else "No commit message"
    return f"{sha} {first_line}"


async def get_single_commit_fallback() -> tuple[str, str]:
    commit_url = f"https://github.com/{settings.github_repository}/commit/{settings.github_sha}"
    output = await run_git("show", "-s", "--format=%B", "HEAD")
    if output.strip():
        msg = _git_commit_record(output.strip())
    else:
        msg = f"{settings.github_sha[:7]} (commit details unavailable)"
    return parse_git_log(msg), commit_url


async def generate_msg_ci() -> str:
    logger.info("Generating Telegram message")

    history_msg: str | None = None
    commit_url: str | None = None
    payload = load_event_payload()

    # 1. Compare the event payload's before commit with the pushed head.
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
            logger.info("Generated message from event payload before-commit comparison")

    # 2. Extract commits directly from the event payload.
    if not history_msg or not commit_url:
        event_commits = get_commits_from_event(payload)
        if event_commits:
            history_msg = parse_git_log("\n".join(event_commits))
            commit_url = get_url_from_event(payload)
            logger.info(
                f"Generated message from {len(event_commits)} commits in event payload"
            )

    # 3. Read the checked-out Git history when the event payload is unavailable.
    if (not history_msg or not commit_url) and settings.github_sha:
        git_log_raw = await get_git_log(f"{settings.github_sha}^", settings.github_sha)
        if git_log_raw:
            history_msg = parse_git_log(git_log_raw)
            commit_url = (
                f"https://github.com/{settings.github_repository}/commit/{settings.github_sha}"
            )

    # 4. Final fallback: current commit details.
    if not history_msg or not commit_url:
        logger.warning("No commit history found locally; falling back to single commit info")
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
