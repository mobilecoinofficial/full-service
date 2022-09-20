#!/usr/bin/python3.9
# Copyright (c) 2021 MobileCoin Inc.
# Copyright (c) 2021 The Forest Team
import functools
import logging
import os
from typing import Optional, cast, Dict


def FuckAiohttp(record: logging.LogRecord) -> bool:
    str_msg = str(getattr(record, "msg", ""))
    if "was destroyed but it is pending" in str_msg:
        return False
    if str_msg.startswith("task:") and str_msg.endswith(">"):
        return False
    return True


logger_class = logging.getLoggerClass()

logger = logging.getLogger()
logger.setLevel("DEBUG")
fmt = logging.Formatter("{levelname} {module}:{lineno}: {message}", style="{")
console_handler = logging.StreamHandler()
console_handler.setLevel(
    ((os.getenv("LOGLEVEL") or os.getenv("LOG_LEVEL")) or "DEBUG").upper()
)
console_handler.setFormatter(fmt)
console_handler.addFilter(FuckAiohttp)
logger.addHandler(console_handler)
logging.getLogger("asyncio").setLevel("INFO")

#### Configure Parameters

# edge cases:
# accessing an unset secret loads other variables and potentially overwrites existing ones
def parse_secrets(secrets: str) -> dict[str, str]:
    pairs = [
        line.strip().split("=", 1)
        for line in secrets.split("\n")
        if line and not line.startswith("#")
    ]
    can_be_a_dict = cast(list[tuple[str, str]], pairs)
    return dict(can_be_a_dict)


# to dump: "\n".join(f"{k}={v}" for k, v in secrets.items())


@functools.cache  # don't load the same env more than once
def load_secrets(env: Optional[str] = None, overwrite: bool = False) -> None:
    if not env:
        env = os.environ.get("ENV", "dev")
    try:
        logging.info("loading secrets from %s_secrets", env)
        secrets = parse_secrets(open(f"{env}_secrets").read())
        if overwrite:
            new_env = secrets
        else:
            # mask loaded secrets with existing env
            new_env = secrets | os.environ
        os.environ.update(new_env)
    except FileNotFoundError:
        pass


secret_cache: Dict[str, str] = {}

# potentially split this into get_flag and get_secret; move all of the flags into fly.toml;
# maybe keep all the tomls and dockerfiles in a separate dir with a deploy script passing --config and --dockerfile explicitly
def get_secret(key: str, env: Optional[str] = None) -> str:
    if key in secret_cache:
        return secret_cache[key]
    try:
        secret = os.environ[key]
    except KeyError:
        load_secrets(env)
        secret = os.environ.get(key) or ""  # fixme
        secret_cache[key] = secret
    if secret.lower() in ("0", "false", "no"):
        return ""
    return secret


## Parameters for easy access and ergonomic use

BASE_CLIENT_PORT = 3200
BASE_PEER_PORT = 3300
BASE_ADMIN_PORT = 3400
BASE_ADMIN_HTTP_GATEWAY_PORT = 3500

# TODO make these command line arguments
IAS_API_KEY = get_secret('IAS_API_KEY')  # 32 bytes
IAS_SPID = get_secret('IAS_SPID')  # 16 bytes
MOBILECOIN_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '../..', 'mobilecoin'))
FULLSERVICE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '../..'))
MOB_RELEASE = get_secret('MOB_RELEASE', '1')
TARGET_DIR = 'target/release'
KEY_DIR = os.path.join(MOBILECOIN_DIR, 'target/sample_data/keys')
WORK_DIR = os.path.join(MOBILECOIN_DIR, TARGET_DIR, 'mc-local-network')
LEDGER_BASE = os.path.join(MOBILECOIN_DIR, 'target', "sample_data", "ledger")
MINTING_KEYS_DIR = os.path.join(WORK_DIR, 'minting-keys')
CLI_PORT = 31337



#### Configure logging to file

if get_secret("LOGFILES"):
    handler = logging.FileHandler("debug.log")
    handler.setLevel("DEBUG")
    handler.setFormatter(fmt)
    handler.addFilter(FuckAiohttp)
    logger.addHandler(handler)
