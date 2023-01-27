from mobilecoin.cli import CommandLineInterface
from mobilecoin.client import (
    Client,
    WalletAPIError,
    mob2pmob,
    pmob2mob,
)

import logging
logging.basicConfig(
    format="{levelname} [{name}:{lineno}] {message}",
    style='{',
    level='ERROR',
)
