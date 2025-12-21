#! /bin/sh

# Regenerate the necessary Python protobuf files.
# Use relative imports.
for PROTO_NAME in printable external
do
    protoc --proto_path=../mobilecoin/api/proto/ ../mobilecoin/api/proto/${PROTO_NAME}.proto --python_out=mobilecoin/util
    sed -i 's/^import \([a-zA-Z_]*_pb2\)/from . import \1/' "mobilecoin/util/${PROTO_NAME}_pb2.py"
done

