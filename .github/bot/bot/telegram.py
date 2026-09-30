# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

from contextlib import ExitStack
import telegram

from . import logger, settings


async def post(msg: str, files: list[str], parse_mode: str):
    logger.info(f"Posting to Telegram (files: {len(files)})")
    async with telegram.Bot(settings.bot_token) as bot:
        if not files:
            logger.info("No files to post, sending message only")
            await bot.send_message(settings.chat_id, text=msg, parse_mode=parse_mode)
        elif len(files) == 1:
            logger.info(f"Sending 1 file with {parse_mode} caption:\n{msg}")
            with open(files[0], "rb") as f:
                await bot.send_document(
                    settings.chat_id, document=f, caption=msg, parse_mode=parse_mode
                )
        else:
            logger.info(f"Sending {len(files)} files with {parse_mode} caption:\n{msg}")
            with ExitStack() as stack:
                file_objs = [stack.enter_context(open(f, "rb")) for f in files]
                medias = [telegram.InputMediaDocument(fo) for fo in file_objs]
                medias[-1] = telegram.InputMediaDocument(
                    file_objs[-1], caption=msg, parse_mode=parse_mode
                )
                await bot.send_media_group(settings.chat_id, medias)
    logger.info("Successfully posted to Telegram")
