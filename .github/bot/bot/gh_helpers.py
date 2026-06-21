# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from typing import cast
from asyncio import sleep

from . import cache, logger, settings
from .github import get_workflow_run, list_workflow_runs  # , compare_commit

# from .parsing import parse_commit_message


async def get_workflow_file() -> str:
    if not cache.workflow_file:
        logger.info("Workflow file not cached, fetching from workflow run")
        run = await get_workflow_run(settings.run_id)
        workflow_path = cast(str, run["path"])
        cache.workflow_file = workflow_path.rsplit("/", 1)[-1].split("@", 1)[0]
        logger.info(f"cached workflow file: {cache.workflow_file}")
    else:
        logger.info(f"Using cached workflow file: {cache.workflow_file}")

    return cache.workflow_file

async def get_last_ci_run(before_run_id: int | None = None, before_commit: str | None = None) -> tuple[dict, bool] | None:
    before = before_run_id or settings.run_id
    logger.info(f"Getting last CI run ID before {before}")
    page = 1
    read = 0
    total = float("inf")
    found_before_at_prior_page = False
    found_commit_at_prior_page = before_commit is None
    while read < total:
        data = await list_workflow_runs(page)
        total = data["total_count"]
        found_before = found_before_at_prior_page
        found_commit = found_commit_at_prior_page
        for run in data["workflow_runs"]:
            ignore_this = False
            if run["id"] == before:
                found_before = True
                logger.info(f"Found before CI run: {run['id']}")
                ignore_this = True
            if run['head_sha'] == before_commit:
                found_commit = True
                logger.info(f"Found before CI commit: {run['head_sha']}")
                ignore_this = True
            if ignore_this:
                continue
            if found_before and found_commit:
                logger.info(f"Found previous CI run: {run['id']} with conclusion {run['conclusion']}")
                if run['conclusion'] == "success":
                    return run, True
                elif not run['conclusion']:
                    return run, False
        else:
            page += 1
            read += len(data["workflow_runs"])
            found_before_at_prior_page = found_before
            found_commit_at_prior_page = found_commit
    return None

async def wait_for_ci_run(last_ci_run_id: int, waiting_max_secs: int = 600) -> dict | None:
    logger.info(f"Waiting for run {last_ci_run_id} to finish")
    elapsed_secs = 0
    next_sleep_secs = 1
    while True:
        run = await get_workflow_run(last_ci_run_id)
        if run["conclusion"]:
            logger.info(f"Run {last_ci_run_id} finished with conclusion {run['conclusion']}")
            if run["conclusion"] == "success":
                return run
            else:
                return None
        
        elapsed_secs += next_sleep_secs
        if elapsed_secs > waiting_max_secs:
            logger.error(f"Waiting for run {last_ci_run_id} to finish for {waiting_max_secs} seconds, giving up")
            return None
        
        await sleep(next_sleep_secs)
        next_sleep_secs *= 2

async def get_last_success_ci_run(before_commit: str | None = None) -> dict | None:
    before_id = settings.run_id
    while True:
        last_ci_run = await get_last_ci_run(before_id, before_commit)
        if not last_ci_run:
            logger.error("No CI run found, giving up")
            return None
        last_ci_run, success = last_ci_run
        before_id = last_ci_run['id']
        if success:
            return last_ci_run
        last_ci_run = await wait_for_ci_run(last_ci_run['id'])
        if last_ci_run:
            return last_ci_run

async def get_last_success_commit(before_commit: str | None = None) -> str | None:
    last_ci_run = await get_last_success_ci_run(before_commit)
    if not last_ci_run:
        logger.error("No last success CI run found, giving up")
        return None
    return last_ci_run["head_sha"]

# async def generate_history(base: str, head: str) -> tuple[str, str]:
#     logger.info(f"Generating commit history between {base} and {head}")
#     msg = ""
#     page = 1
#     proceed_commits = 0
#     total_commits = float("inf")
#     while proceed_commits < total_commits:
#         data = await compare_commit(base, head, page)
#         total_commits = data["total_commits"]
#         for commit in data["commits"]:
#             len_msgs = len(msg)
#             proceed_commits += 1
#             msg += f"{parse_commit_message(commit['commit']['message'])}\n\n---\n\n"
#             if len(msg) >= 512:
#                 msg = msg[:len_msgs]
#                 proceed_commits -= 1
#                 msg += f"{total_commits - proceed_commits} more commits"
#                 logger.info(
#                     f"Generated commit history (truncated) with {proceed_commits} commits"
#                 )
#                 return data["html_url"], msg
#         page += 1
#     if not msg:
#         msg = "No commits found???"
#         logger.warning("No commits found in history")
#     else:
#         msg = msg[:-7]  # remove tail
#         logger.info(f"Generated commit history with {proceed_commits} commits")
#     return data["html_url"], msg
