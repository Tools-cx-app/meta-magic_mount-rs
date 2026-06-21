# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from logging import basicConfig
from asyncio import run
from . import main

basicConfig(
    level="INFO",
    format="%(asctime)s - %(levelname)s - %(message)s",
)

run(main())
