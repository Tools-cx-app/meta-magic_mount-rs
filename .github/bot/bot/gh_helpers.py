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


async def get_last_success_ci_commit() -> str | None:
    logger.info("Getting last successful CI commit")
    page = 1
    read = 0
    total = float("inf")
    found_this_at_prior_page = False
    wait_time = 1
    while read < total:
        data = await list_workflow_runs(page)
        total = data["total_count"]
        read += len(data["workflow_runs"])
        found_this = found_this_at_prior_page
        for run in data["workflow_runs"]:
            if run["id"] == settings.run_id:
                found_this = True
                logger.info("Found this CI run.")
                continue
            if found_this:
                if not run["conclusion"]:
                    logger.info(
                        f"CI run {run['id']} is not completed. Waiting {wait_time} seconds..."
                    )
                    await sleep(wait_time)
                    wait_time *= 2
                    break
                if run["conclusion"] == "success":
                    logger.info(f"Found last successful CI commit: {run['head_sha']}")
                    return run["head_sha"]
        else:
            page += 1
            found_this_at_prior_page = True
    logger.warning("No successful CI commit found")
    return None


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
