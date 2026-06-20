# Copyright (C) 2026 meta-magic_mount-rs developers
# SPDX-License-Identifier: Apache-2.0

from base64 import b64encode
from glob import glob
from nacl import encoding, public

from . import logger
from .config import GH_CI_DIST_PATTERN


def encrypt(public_key: str, secret_value: str) -> str:
    logger.info("Encrypting secret value")
    public_key_obj = public.PublicKey(
        public_key.encode("utf-8"),
        encoding.Base64Encoder,
    )
    sealed_box = public.SealedBox(public_key_obj)
    encrypted = sealed_box.encrypt(secret_value.encode("utf-8"))
    encrypted_value = b64encode(encrypted).decode("utf-8")
    logger.info("Successfully encrypted secret value")
    return encrypted_value


def get_dist() -> list[str]:
    logger.info(f"Getting distribution files with pattern: {GH_CI_DIST_PATTERN}")
    files = glob(GH_CI_DIST_PATTERN)
    logger.info(f"Found {len(files)} distribution files")
    return files
