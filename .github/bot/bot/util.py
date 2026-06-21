# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from glob import glob

from . import logger
from .config import GH_CI_DIST_PATTERN


def get_dist() -> list[str]:
    logger.info(f"Getting distribution files with pattern: {GH_CI_DIST_PATTERN}")
    files = glob(GH_CI_DIST_PATTERN)
    logger.info(f"Found {len(files)} distribution files")
    return files
