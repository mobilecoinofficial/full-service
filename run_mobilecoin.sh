SGX_SPID=9E933DB667EDDE8454258EF3CDC6C2BC
SGX_PRIMARY_KEY=8e98028f29e6472ea4a9f1933c7fc093
SGX_SECONDARY_KEY=acf143a705a249728a4665633a932038
MC_SEED=a4aa76e4a5ca70c8447dd544a63f180b5a6fe0aff96495802506354c10f2886e
SGX_MODE=HW
IAS_MODE=PROD
COMMIT_HASH=22d3894b6b556eb361167b121b934565d5d87c5d

# run docker
if ! [ -x "$(command -v docker)" ]; then
  echo 'Error: docker is not installed.' >&2
  exit 1
fi
sudo systemctl start docker

# todo: detect if docker daemon starts
echo "checking docker status"
i=0
while [ $i -lt 15 ]
do
    if sudo docker info > /dev/null 2>&1; then
        echo "docker daemon is running"
        break
    fi
    sleep 1
    i=`expr $i + 1`
done

if ! sudo docker info > /dev/null 2>&1; then
    echo "docker daemon is not running"
    exit 1
fi

# fetch correct github repo
if ! [ -x "$(command -v git)" ]; then
  echo 'Error: git is not installed.' >&2
  exit 1
fi
if [ ! -d "$(pwd)"/mobilecoin ]; then
    sudo git clone https://github.com/mobilecoinfoundation/mobilecoin
fi
sudo git -C "$(pwd)"/mobilecoin checkout $COMMIT_HASH


    
# run mob
cd mobilecoin
sudo ./mob prompt \
mobilecoin \
--expose 3200 3201 3202 \
--publish 3200 3201 3202 \
--command "cd tools/local-network && ls && ./bootstrap.sh && export MC_SEED=a4aa76e4a5ca70c8447dd544a63f180b5a6fe0aff96495802506354c10f2886e && export LEDGER_BASE=/tmp/mobilenode/target/sample_data/ledger && export IAS_API_KEY=8e98028f29e6472ea4a9f1933c7fc093 && export IAS_SPID=9E933DB667EDDE8454258EF3CDC6C2BC && python3 local_network.py --network-type a-b-c"

## required in the mobilecoin repo ./mob file:
# (near the top)
# parser.add_argument("--command", nargs='+', default=None, help="Any additional commands to be run in the docker container on startup.")

# (near the bottom)
# if args.command is not None:
#     joined_commands = ' '.join(args.command)
#     docker_run.extend(["-c", joined_commands])