# Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
# SPDX-License-Identifier: Apache-2.0

# Constants
TG_API_ID = 611335
TG_API_HASH = "d524b414d21f4d37f08684c1df41ac9c"
TG_MSG_TEMPLATE_CI = """
New push to Github
<pre>
{commit_message}
</pre>
See commit detail <a href="{commit_url}">here</a>
<a href="https://github.com/{github_repository}/actions/runs/{run_id}">#ci_{run_no}</a>
""".strip()
TG_MSG_EXPECTED_PARSE_MODE_CI = "html"
TG_MSG_TEMPLATE_RELEASE = """
New release available: **{name}**

{body}

[Detail]({url})
"""
TG_MSG_EXPECTED_PARSE_MODE_RELEASE = "markdown"
GH_BASE_URL = "https://api.github.com/repos/"
GH_CI_DIST_PATTERN = "./output/*.zip"
COMMIT_TITLE_MAX_LEN = 64
COMMIT_BODY_MAX_LEN = 128
RELEASE_NOTE_MAX_LEN = 2048


# Standard imports
from base64 import b64encode
from collections.abc import Awaitable
from glob import glob
from logging import basicConfig, getLogger
from typing import cast
from textwrap import shorten
from time import sleep

# Third-party imports
from telethon import TelegramClient
from telethon.sessions import StringSession
from httpx import AsyncClient
from nacl import encoding, public
from pydantic import Field
from pydantic_settings import BaseSettings

# Configure logging
basicConfig(
    level="INFO",
    format="%(levelname)s - %(message)s",
)

logger = getLogger("bot")


# Environment variables
class Settings(BaseSettings):
    bot_token: str
    chat_id: int
    run_no: int
    run_id: int
    bot_ci_session: str | None = None
    github_repository: str
    github_token: str
    github_sha: str
    persist_token: str | None = None
    export_session: bool = False
    is_release: bool = False


# Global state cache
class Cache:
    workflow_file: str | None = None


settings = Settings()  # pyright: ignore[reportCallIssue]

# Global variables
client = AsyncClient()


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
    logger.info(f"Making {method} request to {url}")
    response = await client.request(
        method=method, url=url, headers=headers, params=params, json=json
    )
    response.raise_for_status()
    logger.info(f"Request to {url} succeeded with status {response.status_code}")
    return response.json()


async def get_workflow_run(run_id: int) -> dict:
    logger.info(f"Getting workflow run: {run_id}")
    data = await github_api(endpoint=f"/actions/runs/{run_id}")
    logger.info(f"Got workflow run: {run_id}")
    return data


async def get_workflow_file() -> str:
    if not Cache.workflow_file:
        logger.info("Workflow file not cached, fetching from workflow run")
        run = await get_workflow_run(settings.run_id)
        workflow_path = cast(str, run["path"])
        Cache.workflow_file = workflow_path.rsplit("/", 1)[-1].split("@", 1)[0]
        logger.info(f"Cached workflow file: {Cache.workflow_file}")
    else:
        logger.info(f"Using cached workflow file: {Cache.workflow_file}")

    return Cache.workflow_file


async def list_workflow_runs(page: int = 1) -> dict:
    logger.info(f"Listing workflow runs (page: {page})")
    return await github_api(
        endpoint=f"/actions/workflows/{await get_workflow_file()}/runs",
        params={"event": "push", "page": page},
    )


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
                    sleep(wait_time)
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


async def get_latest_release() -> dict:
    logger.info("Getting latest release")
    data = await github_api(endpoint="/releases/latest")
    logger.info(f"Got latest release: {data.get('tag_name', 'unknown')}")
    return data

def parse_release_body(body: str) -> str:
    logger.info("Parsing release body")
    lines = list(map(str.strip, body.split("\n")))
    for line in lines:
        if line.startswith("##"):
            line = f'**{line[2:]}**'
    parsed = shorten("\n".join(lines), RELEASE_NOTE_MAX_LEN, placeholder="...")
    logger.info(f"Parsed body: {parsed}")
    return parsed

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


async def compare_commit(base: str, head: str, page: int = 1) -> dict:
    logger.info(f"Comparing commits: {base}...{head} (page: {page})")
    return await github_api(endpoint=f"/compare/{base}...{head}", params={"page": page})


def parse_commit_message(msg: str) -> str:
    logger.info("Parsing commit message")
    msg = msg.replace("<", "&lt;").replace(">", "&gt;") + "\n\n"
    title, body = msg.split("\n\n", 1)
    title = shorten(title, COMMIT_TITLE_MAX_LEN, placeholder="...")
    body = shorten(body, COMMIT_BODY_MAX_LEN, placeholder="...")
    parsed = f"{title}\n\n{body}".strip()
    logger.info(f"Parsed message: {parsed}")
    return parsed


async def generate_history(base: str, head: str) -> tuple[str, str]:
    logger.info(f"Generating commit history between {base} and {head}")
    msg = ""
    page = 1
    proceed_commits = 0
    total_commits = float("inf")
    while proceed_commits < total_commits:
        data = await compare_commit(base, head, page)
        total_commits = data["total_commits"]
        for commit in data["commits"]:
            len_msgs = len(msg)
            proceed_commits += 1
            msg += f"{parse_commit_message(commit['commit']['message'])}\n\n---\n\n"
            if len(msg) >= 512:
                msg = msg[:len_msgs]
                proceed_commits -= 1
                msg += f"{total_commits - proceed_commits} more commits"
                logger.info(
                    f"Generated commit history (truncated) with {proceed_commits} commits"
                )
                return data["html_url"], msg
        page += 1
    if not msg:
        msg = "No commits found???"
        logger.warning("No commits found in history")
    else:
        msg = msg[:-7]  # remove tail
        logger.info(f"Generated commit history with {proceed_commits} commits")
    return data["html_url"], msg


async def generate_msg_ci() -> str:
    logger.info("Generating Telegram message")
    base_hash = await get_last_success_ci_commit()
    if base_hash is None:
        logger.warning("No last success CI commit found, cannot generate message")
        return "No last success CI commit found???"
    commit_url, history_msg = await generate_history(base_hash, settings.github_sha)
    message = TG_MSG_TEMPLATE_CI.format(
        commit_message=history_msg.strip(),
        commit_url=commit_url,
        run_no=settings.run_no,
        run_id=settings.run_id,
        github_repository=settings.github_repository,
    )
    logger.info("Generated Telegram message")
    return message


async def get_public_key() -> tuple[str, str]:
    logger.info("Getting GitHub public key for secrets encryption")
    data = await github_api(endpoint="/actions/secrets/public-key")
    logger.info(f"Got public key with ID: {data['key_id']}")
    return data["key_id"], data["key"]


def encrypt(public_key: str, secret_value: str) -> str:
    logger.info("Encrypting secret value")
    public_key_obj = public.PublicKey(
        public_key.encode("utf-8"),
        encoding.Base64Encoder(),  # pyright: ignore[reportArgumentType]
    )
    sealed_box = public.SealedBox(public_key_obj)
    encrypted = sealed_box.encrypt(secret_value.encode("utf-8"))
    encrypted_value = b64encode(encrypted).decode("utf-8")
    logger.info("Successfully encrypted secret value")
    return encrypted_value


async def set_secret(name: str, value: str):
    logger.info(f"Setting GitHub secret: {name}")
    kid, key = await get_public_key()
    encrypted_value = encrypt(key, value)
    await github_api(
        endpoint=f"/actions/secrets/{name}",
        method="PUT",
        json={"encrypted_value": encrypted_value, "key_id": kid},
    )
    logger.info(f"Successfully set GitHub secret: {name}")


async def persist_tg_session(session: str):
    if settings.export_session:
        logger.warning(f"Exporting session: {session}")
    if not settings.persist_token:
        logger.info("persist_token not set, skipping session persistence")
        return
    logger.info("Persisting Telegram session to GitHub secrets")
    await set_secret("BOT_CI_SESSION", session)
    logger.info("Successfully persisted Telegram session")


def get_dist() -> list[str]:
    logger.info(f"Getting distribution files with pattern: {GH_CI_DIST_PATTERN}")
    files = glob(GH_CI_DIST_PATTERN)
    logger.info(f"Found {len(files)} distribution files")
    return files


async def post(msg: str, files: list[str], parse_mode: str):
    logger.info(f"Posting to Telegram (files: {len(files)})")
    bot: TelegramClient = await cast(
        Awaitable,
        TelegramClient(
            StringSession(cast(str, settings.bot_ci_session)),
            TG_API_ID,
            TG_API_HASH,
        ).start(bot_token=settings.bot_token),
    )
    async with bot:
        if not settings.bot_ci_session:
            logger.info("No session string found, exporting and persisting new session")
            await persist_tg_session(bot.session.save())  # type: ignore
        if not files:
            logger.info("No files to post, sending message only")
            await bot.send_message(settings.chat_id, msg, parse_mode=parse_mode)
        else:
            logger.info(f"Sending {len(files)} files with caption: {msg}")
            caption = [""] * len(files)
            caption[-1] = msg
            await bot.send_file(
                settings.chat_id, files, caption=caption, parse_mode=parse_mode
            )
    logger.info("Successfully posted to Telegram")


async def main():
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


if __name__ == "__main__":
    import asyncio

    logger.info("Starting bot script")
    asyncio.run(main())
    logger.info("Bot script completed")
