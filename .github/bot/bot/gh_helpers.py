# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

import json
import os
from pathlib import Path
from typing import cast
from asyncio import sleep

from . import cache, logger, settings
from .github import get_workflow_run, list_workflow_runs, compare_commit


def load_event_payload() -> dict:
    event_path = settings.github_event_path or os.environ.get("GITHUB_EVENT_PATH")
    if not event_path:
        logger.info("No GITHUB_EVENT_PATH configured")
        return {}
    path = Path(event_path)
    if not path.is_file():
        logger.warning(f"GITHUB_EVENT_PATH does not point to a valid file: {event_path}")
        return {}
    try:
        with open(path, "r", encoding="utf-8") as f:
            data = json.load(f)
            logger.info(f"Loaded event payload from {event_path} (keys: {list(data.keys())})")
            return data
    except Exception as e:
        logger.warning(f"Failed to read event payload from {event_path}: {e}")
        return {}


def get_commits_from_event(payload: dict) -> list[str]:
    commits = payload.get("commits", [])
    msgs: list[str] = []
    for commit in commits:
        sha = commit.get("id", "")
        message = commit.get("message", "")
        if sha and message:
            first_line = message.splitlines()[0]
            msgs.append(f"{sha[:7]} {first_line}")
    if not msgs and payload.get("head_commit"):
        head = payload["head_commit"]
        sha = head.get("id", "")
        message = head.get("message", "")
        if sha and message:
            first_line = message.splitlines()[0]
            msgs.append(f"{sha[:7]} {first_line}")
    # Return in newest-first order matching get_git_log
    return list(reversed(msgs))


def get_url_from_event(payload: dict) -> str:
    compare_url = payload.get("compare")
    before = payload.get("before")
    is_initial_push = before is not None and set(before) == {"0"}
    if compare_url and not is_initial_push and "000000000000" not in compare_url:
        return compare_url
    if payload.get("head_commit", {}).get("url"):
        return payload["head_commit"]["url"]
    repo = payload.get("repository", {}).get("full_name") or settings.github_repository
    sha = payload.get("after") or settings.github_sha
    return f"https://github.com/{repo}/commit/{sha}"


async def get_workflow_file() -> str:
    if not cache.workflow_file:
        workflow_ref = os.environ.get("GITHUB_WORKFLOW_REF")
        if workflow_ref:
            filename = workflow_ref.split("@", 1)[0].rsplit("/", 1)[-1]
            cache.workflow_file = filename
            logger.info(f"Extracted workflow file from GITHUB_WORKFLOW_REF: {cache.workflow_file}")
            return cache.workflow_file

        logger.info("Workflow file not cached, fetching from workflow run")
        try:
            run = await get_workflow_run(settings.run_id)
            workflow_path = cast(str, run.get("path", ""))
            if workflow_path:
                cache.workflow_file = workflow_path.rsplit("/", 1)[-1].split("@", 1)[0]
                logger.info(f"cached workflow file: {cache.workflow_file}")
                return cache.workflow_file
        except Exception as e:
            logger.warning(f"Failed to fetch workflow run {settings.run_id}: {e}")

        cache.workflow_file = "release.yml" if settings.is_release else "ci.yml"
        logger.info(f"Using default workflow file: {cache.workflow_file}")
    else:
        logger.info(f"Using cached workflow file: {cache.workflow_file}")

    return cache.workflow_file


async def get_last_ci_run(
    before_run_id: int | None = None,
    before_commit: str | None = None,
    branch: str | None = None,
) -> tuple[dict, bool] | None:
    before = before_run_id or settings.run_id
    logger.info(f"Getting last CI run before id {before}, commit {before_commit}, branch {branch}")
    page = 1
    read = 0
    total = float("inf")
    while read < total:
        data = await list_workflow_runs(page=page, branch=branch)
        runs = data.get("workflow_runs", [])
        if not runs:
            logger.info(f"No workflow runs found on page {page}, stopping search")
            break
        total = data.get("total_count", 0)
        for run in runs:
            if run.get("event") not in ("push", "workflow_dispatch"):
                continue
            if run["id"] >= before:
                continue
            if before_commit and run.get("head_sha") == before_commit:
                continue

            conclusion = run.get("conclusion")
            status = run.get("status")
            logger.info(
                f"Found previous CI run: {run['id']} (status: {status}, conclusion: {conclusion})"
            )
            if conclusion == "success":
                return run, True
            elif not conclusion or status in ("in_progress", "queued", "waiting", "pending"):
                return run, False

        page += 1
        read += len(runs)
    return None


async def wait_for_ci_run(
    last_ci_run_id: int, waiting_max_secs: int = 600
) -> dict | None:
    logger.info(f"Waiting for run {last_ci_run_id} to finish")
    elapsed_secs = 0
    next_sleep_secs = 1
    while True:
        run = await get_workflow_run(last_ci_run_id)
        conclusion = run.get("conclusion")
        if conclusion:
            logger.info(
                f"Run {last_ci_run_id} finished with conclusion {conclusion}"
            )
            if conclusion == "success":
                return run
            else:
                return None

        elapsed_secs += next_sleep_secs
        if elapsed_secs > waiting_max_secs:
            logger.error(
                f"Waiting for run {last_ci_run_id} to finish for {waiting_max_secs} seconds, giving up"
            )
            return None

        await sleep(next_sleep_secs)
        next_sleep_secs *= 2


async def get_last_success_ci_run(
    before_commit: str | None = None,
    branch: str | None = None,
) -> dict | None:
    before_id = settings.run_id
    while True:
        last_ci_run_raw = await get_last_ci_run(
            before_run_id=before_id,
            before_commit=before_commit,
            branch=branch,
        )
        if not last_ci_run_raw:
            logger.error("No CI run found, giving up")
            return None
        last_ci_run, success = last_ci_run_raw
        before_id = last_ci_run["id"]
        if success:
            return last_ci_run
        last_ci_run_x = await wait_for_ci_run(last_ci_run["id"])
        if last_ci_run_x:
            return last_ci_run_x


async def get_last_success_commit(
    before_commit: str | None = None,
    branch: str | None = None,
) -> str | None:
    last_ci_run = await get_last_success_ci_run(before_commit=before_commit, branch=branch)
    if not last_ci_run:
        logger.error("No last success CI run found, giving up")
        return None
    return last_ci_run.get("head_sha")


async def get_git_log(base: str, head: str) -> str:
    logger.info(f"Getting commit messages between {base} and {head}")
    if base == head:
        logger.info("Base and head are identical, skipping git log comparison")
        return ""
    total = float("inf")
    msgs: list[str] = []
    page = 1
    try:
        while len(msgs) < total:
            data = await compare_commit(base, head, page=page)
            total = data.get("total_commits", 0)
            commits = data.get("commits", [])
            if not commits:
                break
            for commit in commits:
                sha = commit.get("sha", "")
                commit_info = commit.get("commit", {})
                message = commit_info.get("message", "")
                first_line = message.splitlines()[0] if message else ""
                msgs.append(f"{sha[:7]} {first_line}")
            page += 1
    except Exception as e:
        logger.warning(f"Failed to get commit comparison between {base} and {head}: {e}")
        return ""
    return "\n".join(reversed(msgs))
