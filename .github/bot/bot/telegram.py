# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

import telegram

from . import logger, settings


async def post(msg: str, files: list[str], parse_mode: str):
    logger.info(f"Posting to Telegram (files: {len(files)})")
    async with telegram.Bot(settings.bot_token) as bot:
        if not files:
            logger.info("No files to post, sending message only")
            await bot.send_message(settings.chat_id, msg, parse_mode)
        elif len(files) == 1:
            logger.info(f"Sending 1 file with {parse_mode} caption:\n{msg}")
            await bot.send_document(
                settings.chat_id, files[0], msg, parse_mode=parse_mode
            )
        else:
            logger.info(f"Sending {len(files)} files with {parse_mode} caption:\n{msg}")
            await bot.send_media_group(
                settings.chat_id,
                [telegram.InputMediaDocument(open(f, "rb")) for f in files],
                caption=msg,
                parse_mode=parse_mode,
            )
    logger.info("Successfully posted to Telegram")
