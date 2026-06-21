# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from httpx import AsyncClient

from . import settings, logger
from .config import GH_BASE_URL


async def github_api(
    endpoint: str,
    params: dict | None = None,
    method: str = "GET",
    json: dict | None = None,
    token: str = settings.github_token,
) -> dict:
    headers = {
        "Authorization": f"Bearer {token}",
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2026-03-10",
    }
    url = GH_BASE_URL + settings.github_repository + endpoint
    async with AsyncClient() as client:
        response = await client.request(
            method=method, url=url, headers=headers, params=params, json=json
        )
    response.raise_for_status()
    return response.json()


async def get_workflow_run(run_id: int) -> dict:
    logger.info(f"Getting workflow run: {run_id}")
    data = await github_api(endpoint=f"/actions/runs/{run_id}")
    logger.info(f"Got workflow run: {run_id}")
    return data


async def list_workflow_runs(page: int = 1) -> dict:
    logger.info(f"Listing workflow runs (page: {page})")
    from .gh_helpers import get_workflow_file

    return await github_api(
        endpoint=f"/actions/workflows/{await get_workflow_file()}/runs",
        params={"event": "push", "page": page},
    )


async def get_latest_release() -> dict:
    logger.info("Getting latest release")
    data = await github_api(endpoint="/releases/latest")
    logger.info(f"Got latest release: {data.get('tag_name', 'unknown')}")
    return data


async def compare_commit(base: str, head: str, page: int = 1) -> dict:
    logger.info(f"Getting commits between {base} and {head}")
    return await github_api(
        endpoint=f"/compare/{base}...{head}",
        params={"page": page},
    )