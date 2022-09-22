import argparse

def parse_network_type_cmd_line_args():
    # pull args from command line
    parser = argparse.ArgumentParser(description='Local network tester')
    parser.add_argument('--network-type', help='Type of network to create', required=True)
    parser.add_argument('--block-version', help='Set the block version argument', type=int)
    return parser.parse_args()
