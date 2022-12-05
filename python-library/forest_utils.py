#!/usr/bin/python3.9
# Copyright (c) 2021 MobileCoin Inc.
# Copyright (c) 2021 The Forest Team

import json
import logging
import os
from typing import Optional, cast, Dict

if __name__ == '__main__':
    module_root = os.getcwd()
else:
    module_root = os.path.dirname(__file__)

def MuteAiohttp(record: logging.LogRecord) -> bool:
    str_msg = str(getattr(record, "msg", ""))
    if "was destroyed but it is pending" in str_msg:
        return False
    if str_msg.startswith("task:") and str_msg.endswith(">"):
        return False
    return True


logger_class = logging.getLoggerClass()

logger = logging.getLogger()
logger.setLevel(((os.getenv("LOGLEVEL") or os.getenv("LOG_LEVEL")) or "DEBUG").upper())
fmt = logging.Formatter("{levelname} {module}:{lineno}: {message}", style="{")
console_handler = logging.StreamHandler()
console_handler.setLevel(
    ((os.getenv("LOGLEVEL") or os.getenv("LOG_LEVEL")) or "DEBUG").upper()
)
console_handler.setFormatter(fmt)
console_handler.addFilter(MuteAiohttp)
logger.addHandler(console_handler)
logging.getLogger("asyncio").setLevel("INFO")

#### Configure Parameters

# edge cases:
# accessing an unset secret loads other variables and potentially overwrites existing ones
def parse_secrets(file: str) -> Dict[str, str]:
    with open(file) as json_file:
        config = json.load(json_file)

    return config


# to dump: "\n".join(f"{k}={v}" for k, v in secrets.items())
env_cache = set()

def load_secrets(env: Optional[str] = None, overwrite: bool = False) -> None:
    if str(env) in env_cache:
        return
    env_cache.add(str(env))
    if not env:
        env = os.environ.get("ENV", "dev")
    try:
        secrets = parse_secrets(f"{module_root}/config")
        if overwrite:
            new_env = secrets
        else:
            # mask loaded secrets with existing env
            new_env = {**secrets, **dict(os.environ)}
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

#URL = os.getenv('URL')

#### Configure logging to file

if get_secret("LOGFILES"):
    handler = logging.FileHandler("debug.log")
    handler.setLevel("DEBUG")
    handler.setFormatter(fmt)
    handler.addFilter(MuteAiohttp)
    logger.addHandler(handler)
