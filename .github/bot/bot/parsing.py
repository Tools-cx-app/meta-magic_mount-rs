# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

import re

from . import logger
from .config import (
    PARSING_MAX_LEN,
)


def titles_to_bold(markdown_text: str) -> str:
    # 正则表达式解析：
    # ^[ \t]*{#}{1,6} : 匹配行首可选的空格，后跟 1 到 6 个 # 号
    # [ \t]+          : 匹配 # 号与标题内容之间的空格
    # (.+?)           : 捕获组 1，即标题的实际文本内容（非贪婪匹配）
    # [ \t]*$         : 匹配行尾可选的空格
    pattern = r"^[ \t]*#{1,6}[ \t]+(.+?)[ \t]*$"

    # re.MULTILINE 确保 ^ 和 $ 能匹配每一行的开头和结尾
    # r'**\1**' 表示将匹配到的整行替换为：** + 捕获组1的内容 + **
    result = re.sub(pattern, r"**\1**", markdown_text, flags=re.MULTILINE)

    return result


def parse_release_body(body: str) -> str:
    logger.info("Parsing release")
    body = titles_to_bold(body)
    if len(body) > PARSING_MAX_LEN:
        body = body[:PARSING_MAX_LEN] + "..."
    logger.info(f"Parsed body: {body}")
    return body


def parse_git_log(log: str) -> str:
    logger.info("Parsing git log")
    lines = log.split("\n")
    parsed = []
    parsed_length = 0
    for line in lines:
        parsed_length += len(line) + 1
        if parsed_length <= PARSING_MAX_LEN:
            parsed.append(line)
        else:
            break
    parsed_str = "\n".join(
        reversed(parsed)
    ).replace("<", "&lt;").replace(">", "&gt;")
    if len(lines) > len(parsed):
        parsed_str = f"...{len(lines)-len(parsed)} more commits...\n" + parsed_str
    logger.info(f"Parsed log: {parsed_str}")
    return parsed_str
