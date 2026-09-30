# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: GPL-v3

import os
from contextlib import ExitStack
import telegram

from . import logger, settings


async def post(msg: str, files: list[str], parse_mode: str):
    logger.info(f"Posting to Telegram (files: {len(files)})")
    valid_files: list[str] = []
    for f in files:
        if not os.path.isfile(f):
            logger.warning(f"File does not exist, skipping: {f}")
        elif os.path.getsize(f) == 0:
            logger.warning(f"File is empty (0 bytes), skipping: {f}")
        else:
            valid_files.append(f)

    async with telegram.Bot(settings.bot_token) as bot:
        if not valid_files:
            logger.info("No files to post, sending message only")
            await bot.send_message(settings.chat_id, text=msg, parse_mode=parse_mode)
        elif len(valid_files) == 1:
            logger.info(f"Sending 1 file with {parse_mode} caption:\n{msg}")
            with open(valid_files[0], "rb") as f:
                await bot.send_document(
                    settings.chat_id, document=f, caption=msg, parse_mode=parse_mode
                )
        else:
            logger.info(f"Sending {len(valid_files)} files with {parse_mode} caption:\n{msg}")
            with ExitStack() as stack:
                medias: list[telegram.InputMediaDocument] = []
                for i, f in enumerate(valid_files):
                    fo = stack.enter_context(open(f, "rb"))
                    caption = msg if i == len(valid_files) - 1 else None
                    parse = parse_mode if caption else None
                    medias.append(
                        telegram.InputMediaDocument(
                            fo, caption=caption, parse_mode=parse
                        )
                    )
                await bot.send_media_group(settings.chat_id, medias)
    logger.info("Successfully posted to Telegram")
