"""
Automatically post new CI to telegram
"""

# Constants
TG_API_ID = 611335
TG_API_HASH = "d524b414d21f4d37f08684c1df41ac9c"
TG_MSG_TEMPLATE = """
New push to Github
```
{commit_message}
```
See commit detail [here]({commit_url})
[#ci_{run_no}](https://github.com/{github_repository}/actions/runs/{run_id})
""".strip()
GH_BASE_URL = "https://api.github.com/repos/"
GH_CI_WORKFLOW_NAME = "ci-build"
GH_CI_DIST_PATTERN = "./output/*.zip"
COMMIT_TITLE_MAX_LEN: int = 64
COMMIT_BODY_MAX_LEN: int = 128


# Standard imports
from base64 import b64encode
from collections.abc import Awaitable
from glob import glob
from logging import basicConfig, getLogger
from pathlib import Path
from typing import cast
from textwrap import shorten

# Third-party imports
from telethon import TelegramClient
from telethon.sessions import StringSession
from httpx import AsyncClient
from nacl import encoding, public
from pydantic_settings import BaseSettings, SettingsConfigDict

# Configure logging
basicConfig(
    level="INFO",
    format="%(levelname)s - %(message)s",
)

logger = getLogger("bot")


# Environment variables
class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=Path(__file__).parent / ".env")
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
    """
    GitHub RESTful API helper function.

    Args:
        endpoint: API endpoint
        params: Query parameters
        method: HTTP method
        json: Request body
        token: GitHub token

    Returns:
        Result of GitHub RESTful API
    """
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


async def list_workflow_runs(page: int = 1) -> dict:
    """
    List workflow runs for the repository.

    Args:
        page: Page number for pagination

    Returns:
        Result of GitHub RESTful API
    """
    logger.info(f"Listing workflow runs (page: {page})")
    return await github_api(
        endpoint="/actions/runs",
        params={"exclude_pull_requests": True, "page": page, "status": "success"},
    )


async def get_last_success_ci_commit() -> str | None:
    """
    Get the last successful CI commit SHA.

    Returns:
        Last successful CI commit SHA or None if not found
    """
    logger.info("Getting last successful CI commit")
    page = 1
    read = 0
    total = float("inf")
    while read < total:
        data = await list_workflow_runs(page)
        total = data["total_count"]
        read += len(data["workflow_runs"])
        for run in data["workflow_runs"]:
            if run["name"] == GH_CI_WORKFLOW_NAME:
                logger.info(f"Found last successful CI commit: {run['head_sha']}")
                return run["head_sha"]
        page += 1
    logger.warning("No successful CI commit found")
    return None


async def compare_commit(base: str, head: str, page: int = 1) -> dict:
    """
    Compare two commits.

    Args:
        base: Base commit SHA
        head: Head commit SHA
        page: Page number for pagination

    Returns:
        Result of GitHub Restful API
    """
    logger.info(f"Comparing commits: {base}...{head} (page: {page})")
    return await github_api(endpoint=f"/compare/{base}...{head}", params={"page": page})


def parse_commit_message(msg: str) -> str:
    """
    Parse commit message to avoid too long message.

    Args:
        msg: Commit message

    Returns:
        Parsed commit message
    """
    msg = msg + "\n\n"
    title, body = msg.split("\n\n", 1)
    title = shorten(title, COMMIT_TITLE_MAX_LEN, placeholder="...")
    body = shorten(body, COMMIT_BODY_MAX_LEN, placeholder="...")
    if not body:
        return title
    return f"{title}\n\n{body}"


async def generate_history(base: str, head: str) -> tuple[str, str]:
    """
    Generate commit history between two commits.

    Args:
        base: Base commit SHA
        head: Head commit SHA

    Returns:
        Tuple of (commit_url, history_message)
    """
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
            msg += f"{parse_commit_message(commit['commit']['message'])}\n\n"
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
        logger.info(f"Generated commit history with {proceed_commits} commits")
    return data["html_url"], msg  # pyright: ignore[reportPossiblyUnboundVariable]


async def generate_msg() -> str:
    """
    Generate Telegram message for CI notification.

    Returns:
        Formatted Telegram message
    """
    logger.info("Generating Telegram message")
    base_hash = await get_last_success_ci_commit()
    if base_hash is None:
        logger.warning("No last success CI commit found, cannot generate message")
        return "No last success CI commit found???"
    commit_url, history_msg = await generate_history(base_hash, settings.github_sha)
    message = TG_MSG_TEMPLATE.format(
        commit_message=history_msg.strip(),
        commit_url=commit_url,
        run_no=settings.run_no,
        run_id=settings.run_id,
        github_repository=settings.github_repository,
    )
    logger.info("Generated Telegram message")
    return message


async def get_public_key() -> tuple[str, str]:
    """
    Get GitHub public key for encrypting secrets.

    Returns:
        Tuple of (key_id, public_key)
    """
    logger.info("Getting GitHub public key for secrets encryption")
    data = await github_api(endpoint="/actions/secrets/public-key")
    logger.info(f"Got public key with ID: {data['key_id']}")
    return data["key_id"], data["key"]


def encrypt(public_key: str, secret_value: str) -> str:
    """
    Encrypt secret value using public key.

    Args:
        public_key: GitHub public key
        secret_value: Secret value to encrypt

    Returns:
        Encrypted secret value
    """
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
    """
    Set GitHub secret.

    Args:
        name: Secret name
        value: Secret value
    """
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
    """
    Persist Telegram session string to GitHub secrets.

    Args:
        session: Telegram session string
    """
    if settings.export_session:
        logger.warning(f"Exporting session: {session}")
    if not settings.persist_token:
        logger.info("persist_token not set, skipping session persistence")
        return
    logger.info("Persisting Telegram session to GitHub secrets")
    await set_secret("BOT_CI_SESSION", session)
    logger.info("Successfully persisted Telegram session")


def get_dist() -> list[str]:
    """
    Get distribution files matching the pattern.

    Returns:
        List of distribution files
    """
    logger.info(f"Getting distribution files with pattern: {GH_CI_DIST_PATTERN}")
    files = glob(GH_CI_DIST_PATTERN)
    logger.info(f"Found {len(files)} distribution files")
    return files


async def post(msg: str, files: list[str] = []):
    """
    Post message and files to Telegram.

    Args:
        msg: Message to send
        files: List of files to send
    """
    logger.info(f"Posting to Telegram (files: {len(files)})")
    bot = TelegramClient(
        StringSession(settings.bot_ci_session),  # pyright: ignore[reportArgumentType]
        TG_API_ID,
        TG_API_HASH,
    ).start(bot_token=settings.bot_token)
    bot: TelegramClient = await cast(Awaitable, bot)
    async with bot:
        if not settings.bot_ci_session:
            logger.info("No session string found, exporting and persisting new session")
            await persist_tg_session(bot.session.save())  # type: ignore
        if not files:
            logger.info("No files to post, sending message only")
            await bot.send_message(settings.chat_id, msg)
        else:
            await bot.send_file(settings.chat_id, files, caption=msg)
    logger.info("Successfully posted to Telegram")


async def main():
    """
    Main function to generate message and post to Telegram.
    """
    logger.info("Starting main function")
    msg = await generate_msg()
    files = get_dist()
    await post(msg, files)
    logger.info("Post done successfully")


if __name__ == "__main__":
    import asyncio

    logger.info("Starting bot script")
    asyncio.run(main())
    logger.info("Bot script completed")
