
import sys
import subprocess

# Path to "cli" modules.
repo_root_dir = subprocess.check_output("git rev-parse --show-toplevel", shell=True).decode("utf8").strip()
sys.path.append("{}/cli".format(repo_root_dir))

from fullservice import FullServiceAPIv2 as v2
from dataobjects import Response, Account  # TODO rename as FSDataObjects


async def main():
    print("Placeholder script")

if __name__ == "__main__":
    main()
