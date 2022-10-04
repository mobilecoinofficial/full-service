# TODO: replace os with pahtlib
import os

BASE_CLIENT_PORT = 3200
BASE_PEER_PORT = 3300
BASE_ADMIN_PORT = 3400
BASE_ADMIN_HTTP_GATEWAY_PORT = 3500

# TODO make these command line arguments
IAS_API_KEY = os.getenv('IAS_API_KEY', default='0' * 64)  # 32 bytes
IAS_SPID = os.getenv('IAS_SPID', default='0' * 32)  # 16 bytes
MOBILECOIN_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '../..', 'mobilecoin'))
FULLSERVICE_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '../..'))
MOB_RELEASE = os.getenv('MOB_RELEASE', '1')
TARGET_DIR = 'target/release'
KEY_DIR = os.path.join(MOBILECOIN_DIR, 'target/sample_data/keys')
WORK_DIR = os.path.join(MOBILECOIN_DIR, TARGET_DIR, 'mc-local-network')
LEDGER_BASE = os.path.join(MOBILECOIN_DIR, 'target', "sample_data", "ledger")
MINTING_KEYS_DIR = os.path.join(WORK_DIR, 'minting-keys')
CLI_PORT = 31337

if not MOB_RELEASE or MOB_RELEASE == '0':
    TARGET_DIR = 'target/debug'

# Sane default log configuration
if 'MC_LOG' not in os.environ:
    os.environ[
        'MC_LOG'] = 'debug,rustls=warn,hyper=warn,tokio_reactor=warn,mio=warn,want=warn,rusoto_core=error,h2=error,reqwest=error,rocket=error,<unknown>=error'
