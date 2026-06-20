# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from collections.abc import Awaitable
from typing import cast
from telethon import TelegramClient
from telethon.sessions import StringSession

from . import logger, settings
from .config import TG_SESSION_SECRET_KEY, TG_API_ID, TG_API_HASH
from .github import set_secret


async def persist_tg_session(session: str):
    if settings.export_session:
        logger.warning(f"Exporting session: {session}")
    if not settings.persist_token:
        logger.info("persist_token not set, skipping session persistence")
        return
    logger.info("Persisting Telegram session to GitHub secrets")
    await set_secret(TG_SESSION_SECRET_KEY, session)
    logger.info("Successfully persisted Telegram session")


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
