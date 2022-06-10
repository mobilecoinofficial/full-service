#!/usr/bin/python3
# Copyright (c) 2018-2022 The MobileCoin Foundation

# TODO
# - Better errors on missing env vars
# - SGX HW/SW
# - Default MC_LOG
import argparse
import json
import os
import shutil
import socketserver
import subprocess
import sys
import threading
import time
from pprint import pformat


BASE_CLIENT_PORT = 3200
BASE_PEER_PORT = 3300
BASE_ADMIN_PORT = 3400
BASE_ADMIN_HTTP_GATEWAY_PORT = 3500
MOBILECOIND_PORT = 4444

# TODO make these command line arguments
LEDGER_BASE = os.path.abspath(os.getenv('LEDGER_BASE'))
IAS_API_KEY = os.getenv('IAS_API_KEY', default='0'*64) # 32 bytes
IAS_SPID = os.getenv('IAS_SPID', default='0'*32) # 16 bytes
PROJECT_DIR = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', 'mobilecoin'))
MOB_RELEASE = os.getenv('MOB_RELEASE', '1')
CARGO_FLAGS = '--release'
TARGET_DIR = 'target/release'
WORK_DIR =  os.path.join(PROJECT_DIR, TARGET_DIR, 'mc-local-network')
MINTING_KEYS_DIR = os.path.join(WORK_DIR, 'minting-keys')
CLI_PORT = 31337

if MOB_RELEASE == '0':
    CARGO_FLAGS = ''
    TARGET_DIR = 'target/debug'

# Sane default log configuration
if 'MC_LOG' not in os.environ:
    os.environ['MC_LOG'] = 'debug,rustls=warn,hyper=warn,tokio_reactor=warn,mio=warn,want=warn,rusoto_core=error,h2=error,reqwest=error,rocket=error,<unknown>=error'


class QuorumSet:
    def __init__(self, threshold, members):
        self.threshold = threshold
        self.members = members

    def resolve_to_json(self, nodes_by_name):
        resolved_members = []
        for member in self.members:
            if isinstance(member, str):
                peer_port = nodes_by_name[member].peer_port
                resolved_members.append({'type': 'Node', 'args': f'localhost:{peer_port}'})
            elif isinstance(member, QuorumSet):
                resolved_members.append({'type': 'InnerSet', 'args': member.resolve_to_json(nodes_by_name)})
            else:
                raise Exception(f'Unsupported member type: {type(member)}')
        return {
            'threshold': self.threshold,
            'members': resolved_members,
        }


class Peer:
    def __init__(self, name, broadcast_consensus_msgs=True):
        self.name = name
        self.broadcast_consensus_msgs = broadcast_consensus_msgs

    def __repr__(self):
        return self.name


class Node:
    def __init__(self, name, node_num, client_port, peer_port, admin_port, admin_http_gateway_port, peers, quorum_set, block_version):
        assert all(isinstance(peer, Peer) for peer in peers)
        assert isinstance(quorum_set, QuorumSet)

        self.name = name
        self.node_num = node_num
        self.client_port = client_port
        self.peer_port = peer_port
        self.admin_port = admin_port
        self.admin_http_gateway_port = admin_http_gateway_port
        self.peers = peers
        self.quorum_set = quorum_set
        self.minimum_fee = 400_000_000
        self.block_version = block_version or 2

        self.consensus_process = None
        self.ledger_distribution_process = None
        self.admin_http_gateway_process = None
        self.ledger_dir = os.path.join(WORK_DIR, f'node-ledger-{self.node_num}')
        self.ledger_distribution_dir = os.path.join(WORK_DIR, f'node-ledger-distribution-{self.node_num}')
        self.msg_signer_key_file = os.path.join(WORK_DIR, f'node-scp-{self.node_num}.pem')
        self.tokens_config_file = os.path.join(WORK_DIR, f'node-tokens-{self.node_num}.json')
        subprocess.check_output(f'openssl genpkey -algorithm ed25519 -out {self.msg_signer_key_file}', shell=True)

    def peer_uri(self, broadcast_consensus_msgs=True):
        pub_key = subprocess.check_output(f'openssl pkey -in {self.msg_signer_key_file} -pubout | head -n-1 | tail -n+2 | sed "s/+/-/g; s/\//_/g"', shell=True).decode().strip()
        broadcast_consensus_msgs = '1' if broadcast_consensus_msgs else '0'
        return f'insecure-mcp://localhost:{self.peer_port}/?consensus-msg-key={pub_key}&broadcast-consensus-msgs={broadcast_consensus_msgs}'

    def __repr__(self):
        return self.name

    def start(self, network):
        assert not self.consensus_process

        if self.ledger_distribution_process:
            self.ledger_distribution_process.terminate()
            self.ledger_distribution_process = None

        if self.admin_http_gateway_process:
            self.admin_http_gateway_process.terminate()
            self.admin_http_gateway_process = None

        # A map of node name -> Node object
        nodes_by_name = {node.name: node for node in network.nodes}

        # Private SCP signing key
        msg_signer_key = subprocess.check_output(f'cat {self.msg_signer_key_file} | head -n-1 | tail -n+2', shell=True).decode().strip()

        # URIs for the peers above
        peer_uris = [nodes_by_name[peer.name].peer_uri(
            broadcast_consensus_msgs=peer.broadcast_consensus_msgs,
        ) for peer in self.peers]

        # URIs for all additional nodes in the network, in case they appear in our quorum set
        peer_names = [peer.name for peer in self.peers]
        known_peers = [node.peer_uri() for node in network.nodes if node.name not in peer_names and node.name != self.name]
        tx_source_urls = [f'file://{node.ledger_distribution_dir}' for node in network.nodes if node.name in peer_names]

        # Our quorum set and associated JSON
        quorum_set = {
            'quorum_set': self.quorum_set.resolve_to_json(nodes_by_name),
            'broadcast_peers': peer_uris,
            'known_peers': known_peers,
            'tx_source_urls': tx_source_urls,
        }
        network_json_path = os.path.join(WORK_DIR, f'node{self.node_num}-network.json')
        with open(network_json_path, 'w') as f:
            json.dump(quorum_set, f)

        try:
            shutil.rmtree(f'{WORK_DIR}/scp-debug-dump-{self.node_num}')
        except FileNotFoundError:
            pass

        # Tokens config file
        tokens_config = {
            "tokens": [
                { "token_id": 0, "minimum_fee": self.minimum_fee },
                {
                    "token_id": 1,
                    "minimum_fee": 1024,
                    "governors": {
                        "signers": open(os.path.join(MINTING_KEYS_DIR, 'governor1.pub')).read(),
                        "threshold": 1
                    }
                },
                {
                    "token_id": 2,
                    "minimum_fee": 1024,
                    "governors": {
                        "signers": open(os.path.join(MINTING_KEYS_DIR, 'governor2.pub')).read(),
                        "threshold": 1
                    }
                },
             ],
        }
        with open(self.tokens_config_file, 'w') as f:
            json.dump(tokens_config, f)

        #  Sign the governors with the admin key.
        subprocess.check_output(' '.join([
            f'cd {PROJECT_DIR} && exec {TARGET_DIR}/mc-consensus-mint-client',
            'sign-governors',
            f'--tokens {self.tokens_config_file}',
            f'--signing-key {MINTING_KEYS_DIR}/minting-trust-root.pem',
            f'--output-json {self.tokens_config_file}',
        ]), shell=True)

        cmd = ' '.join([
            f'cd {PROJECT_DIR} && exec {TARGET_DIR}/consensus-service',
            f'--client-responder-id localhost:{self.client_port}',
            f'--peer-responder-id localhost:{self.peer_port}',
            f'--msg-signer-key "{msg_signer_key}"',
            f'--network {network_json_path}',
            f'--ias-api-key={IAS_API_KEY}',
            f'--ias-spid={IAS_SPID}',
            f'--origin-block-path {LEDGER_BASE}',
            f'--block-version {self.block_version}',
            f'--ledger-path {self.ledger_dir}',
            f'--admin-listen-uri="insecure-mca://0.0.0.0:{self.admin_port}/"',
            f'--client-listen-uri="insecure-mc://0.0.0.0:{self.client_port}/"',
            f'--peer-listen-uri="insecure-mcp://0.0.0.0:{self.peer_port}/"',
            f'--scp-debug-dump {WORK_DIR}/scp-debug-dump-{self.node_num}',
            f'--sealed-block-signing-key {WORK_DIR}/consensus-sealed-block-signing-key-{self.node_num}',
            f'--tokens={self.tokens_config_file}',
        ])

        print(f'Starting node {self.name}: client_port={self.client_port} peer_port={self.peer_port} admin_port={self.admin_port}')
        print(f' - Peers: {self.peers}')
        print(f' - Quorum set: {pformat(quorum_set)}')
        print(cmd)
        print()

        self.consensus_process = subprocess.Popen(cmd, shell=True)

        # Wait for ledger db to become available
        ledger_db = os.path.join(self.ledger_dir, 'data.mdb')
        while not os.path.exists(ledger_db):
            if self.consensus_process.poll() is not None:
                print('consensus process crashed')
                return self.stop()
            print(f'Waiting for {ledger_db}')
            time.sleep(1)

        cmd = ' '.join([
            f'cd {PROJECT_DIR} && exec {TARGET_DIR}/ledger-distribution',
            f'--ledger-path {self.ledger_dir}',
            f'--dest "file://{self.ledger_distribution_dir}"',
            f'--state-file {WORK_DIR}/ledger-distribution-state-{self.node_num}',
        ])
        print(f'Starting local ledger distribution: {cmd}')
        self.ledger_distribution_process = subprocess.Popen(cmd, shell=True)

        cmd = ' '.join([
            f'cd {PROJECT_DIR} && export ROCKET_CLI_COLORS=0 && exec {TARGET_DIR}/mc-admin-http-gateway',
            f'--listen-host 0.0.0.0',
            f'--listen-port {self.admin_http_gateway_port}',
            f'--admin-uri insecure-mca://127.0.0.1:{self.admin_port}/',
        ])
        print(f'Starting admin http gateway: {cmd}')
        self.admin_http_gateway_process = subprocess.Popen(cmd, shell=True)

    def status(self):
        if not self.consensus_process:
            return 'stopped'

        if self.consensus_process.poll() is not None:
            return 'exited'

        return f'running, pid={self.consensus_process.pid}'

    def stop(self):
        if self.consensus_process and self.consensus_process.poll() is None:
            self.consensus_process.terminate()
            self.consensus_process = None

        if self.ledger_distribution_process and self.ledger_distribution_process.poll() is None:
            self.ledger_distribution_process.terminate()
            self.ledger_distribution_process = None

        if self.admin_http_gateway_process and self.admin_http_gateway_process.poll() is None:
            self.admin_http_gateway_process.terminate()
            self.admin_http_gateway_process = None

        print(f'Stopped node {self}!')


class NetworkCLI(threading.Thread):
    """Network command line interface (over TCP)"""
    def __init__(self, network):
        super().__init__()
        self.network = network
        self.server = None

    def run(self):
        network = self.network

        class NetworkCLITCPHandler(socketserver.StreamRequestHandler):
            def send(self, s):
                self.wfile.write(bytes(s, 'utf-8'))

            def handle(self):
                self.send('> ')
                while True:
                    try:
                        line = self.rfile.readline().strip().decode()
                    except:
                        return

                    if not line:
                        continue

                    if ' ' in line:
                        cmd, args = line.split(' ', 1)
                    else:
                        cmd = line
                        args = ''

                    if cmd == 'status':
                        for node in network.nodes:
                            self.send(f'{node.name}: {node.status()}\n')

                    elif cmd == 'stop':
                        node = network.get_node(args)
                        if node:
                            node.stop()
                            self.send(f'Stopped {args}.\n')
                        else:
                            self.send(f'Unknown node {args}\n')

                    elif cmd == 'start':
                        node = network.get_node(args)
                        if node:
                            node.stop()
                            node.start(network)
                            self.send(f'Started {args}.\n')
                        else:
                            self.send(f'Unknown node {args}\n')


                    else:
                        self.send('Unknown command\n')

                    self.send('> ')

        assert self.server is None
        socketserver.TCPServer.allow_reuse_address = True
        self.server = socketserver.TCPServer(('0.0.0.0', CLI_PORT), NetworkCLITCPHandler)
        self.server.serve_forever()

    def stop(self):
        self.server.shutdown()

class Network:
    def __init__(self):
        self.nodes = []
        self.ledger_distribution = None
        self.cli = None
        try:
            shutil.rmtree(WORK_DIR)
        except FileNotFoundError:
            pass
        os.mkdir(WORK_DIR)

    def build_binaries(self):
        print('Building binaries...')
        enclave_pem = os.path.join(PROJECT_DIR, 'Enclave_private.pem')
        if not os.path.exists(enclave_pem):
            subprocess.run(
                f'openssl genrsa -out {enclave_pem} -3 3072',
                shell=True,
                check=True,
            )

        subprocess.run(
            f'cd {PROJECT_DIR} && CONSENSUS_ENCLAVE_PRIVKEY="{enclave_pem}" cargo build -p mc-consensus-service -p mc-ledger-distribution -p mc-admin-http-gateway -p mc-util-grpc-admin-tool -p mc-mint-auditor -p mc-crypto-x509-test-vectors -p mc-consensus-mint-client -p mc-util-seeded-ed25519-key-gen {CARGO_FLAGS}',
            shell=True,
            check=True,
        )

    def add_node(self, name, peers, quorum_set):
        node_num = len(self.nodes)
        self.nodes.append(Node(
            name,
            node_num,
            BASE_CLIENT_PORT + node_num,
            BASE_PEER_PORT + node_num,
            BASE_ADMIN_PORT + node_num,
            BASE_ADMIN_HTTP_GATEWAY_PORT + node_num,
            peers,
            quorum_set,
            self.block_version,
        ))

    def get_node(self, name):
        for node in self.nodes:
            if node.name == name:
                return node

    def generate_minting_keys(self):
       os.mkdir(MINTING_KEYS_DIR)

       subprocess.check_output(f'openssl genpkey -algorithm ed25519 -out {MINTING_KEYS_DIR}/governor1', shell=True)
       subprocess.check_output(f'openssl pkey -pubout -in {MINTING_KEYS_DIR}/governor1 -out {MINTING_KEYS_DIR}/governor1.pub', shell=True)

       subprocess.check_output(f'openssl genpkey -algorithm ed25519 -out {MINTING_KEYS_DIR}/governor2', shell=True)
       subprocess.check_output(f'openssl pkey -pubout -in {MINTING_KEYS_DIR}/governor2 -out {MINTING_KEYS_DIR}/governor2.pub', shell=True)

       # This matches the hardcoded key in consensus/enclave/impl/build.rs
       subprocess.check_output(f'cd {PROJECT_DIR} && exec {TARGET_DIR}/mc-util-seeded-ed25519-key-gen --seed abababababababababababababababababababababababababababababababab > {MINTING_KEYS_DIR}/minting-trust-root.pem', shell=True)

    def start(self):
        self.stop()

        print("Generating minting keys")
        self.generate_minting_keys()

        print("Starting nodes")
        for node in self.nodes:
            node.start(self)

        print("Starting network CLI")
        self.cli = NetworkCLI(self)
        self.cli.start()


    def wait(self):
        """Block until one of our processes dies."""
        while True:
            for node in self.nodes:
                if node.consensus_process and node.consensus_process.poll() is not None:
                    print(f'Node {node} consensus service died with exit code {node.consensus_process.poll()}')
                    return False

                if node.admin_http_gateway_process and node.admin_http_gateway_process.poll() is not None:
                    print(f'Node {node} admin http gateway died with exit code {node.admin_http_gateway_process.poll()}')
                    return False

                if node.ledger_distribution_process and node.ledger_distribution_process.poll() is not None:
                    print(f'Node {node} ledger distribution died with exit code {node.ledger_distribution_process.poll()}')
                    return False

            if self.mobilecoind.process and self.mobilecoind.process.poll() is not None:
                print(f'mobilecoind died with exit code {self.mobilecoind.process.poll()}')
                return False

            time.sleep(1)

    def stop(self):
        if self.cli is not None:
            self.cli.stop()
            self.cli = None

        # print("Killing any existing processes")
        # try:
        #     subprocess.check_output("killall -9 consensus-service filebeat ledger-distribution prometheus mc-admin-http-gateway mobilecoind 2>/dev/null", shell=True)
        # except subprocess.CalledProcessError as exc:
        #     if exc.returncode != 1:
        #         raise


    def default_entry_point(self, network_type, skip_build=False, block_version=None):
        self.block_version = block_version

        if network_type == 'dense5':
            #  5 node interconnected network requiring 4 out of 5  nodes.
            num_nodes = 5
            for i in range(num_nodes):
                other_nodes = [str(j) for j in range(num_nodes) if i != j]
                peers = [Peer(p) for p in other_nodes]
                self.add_node(str(i), peers, QuorumSet(3, other_nodes))

        elif network_type == 'a-b-c':
            # 3 nodes, where all 3 are required but node `a` and `c` are not peered together.
            # (i.e. a <-> b <-> c)
            self.add_node('a', [Peer('b')], QuorumSet(2, ['b', 'c']))
            self.add_node('b', [Peer('a'), Peer('c')], QuorumSet(2, ['a', 'c']))
            self.add_node('c', [Peer('b')], QuorumSet(2, ['a', 'b']))

        elif network_type == 'ring5':
            # A ring of 5 nodes where each node:
            # - sends SCP messages to the node before it and after it
            # - has the node after it in its quorum set
            self.add_node('1', [Peer('5'), Peer('2')], QuorumSet(1, ['2']))
            self.add_node('2', [Peer('1'), Peer('3')], QuorumSet(1, ['3']))
            self.add_node('3', [Peer('2'), Peer('4')], QuorumSet(1, ['4']))
            self.add_node('4', [Peer('3'), Peer('5')], QuorumSet(1, ['5']))
            self.add_node('5', [Peer('4'), Peer('1')], QuorumSet(1, ['1']))

        elif network_type == 'ring5b':
            # A ring of 5 nodes where each node:
            # - sends SCP messages to the node after it
            # - has the node after it in its quorum set
            self.add_node('1', [Peer('5', broadcast_consensus_msgs=False), Peer('2')], QuorumSet(1, ['2']))
            self.add_node('2', [Peer('1', broadcast_consensus_msgs=False), Peer('3')], QuorumSet(1, ['3']))
            self.add_node('3', [Peer('2', broadcast_consensus_msgs=False), Peer('4')], QuorumSet(1, ['4']))
            self.add_node('4', [Peer('3', broadcast_consensus_msgs=False), Peer('5')], QuorumSet(1, ['5']))
            self.add_node('5', [Peer('4', broadcast_consensus_msgs=False), Peer('1')], QuorumSet(1, ['1']))

        else:
            raise Exception('Invalid network type')

        if not skip_build:
            self.build_binaries()

        self.start()
        self.wait()
        self.stop()

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description='Local network tester')
    parser.add_argument('--network-type', help='Type of network to create', required=True)
    parser.add_argument('--skip-build', help='Skip building binaries', action='store_true')
    parser.add_argument('--block-version', help='Set the block version argument', type=int)
    args = parser.parse_args()

    Network().default_entry_point(args.network_type, args.skip_build, args.block_version)
