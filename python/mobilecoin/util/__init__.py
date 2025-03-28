from decimal import Decimal
import zlib
import base64

import base58

from . import external_pb2
from . import printable_pb2


PMOB = Decimal("1e12")


def mob2pmob(x):
    """Convert from MOB to picoMOB."""
    return round(Decimal(x) * PMOB)


def pmob2mob(x):
    """Convert from picoMOB to MOB."""
    result = int(x) / PMOB
    if result == 0:
        return Decimal("0")
    else:
        return result


def b64_public_address_to_b58_wrapper(b64_string):
    """Convert a b64-encoded PublicAddress protobuf to a b58-encoded PrintableWrapper protobuf"""
    public_address_bytes = base64.b64decode(b64_string)

    public_address = external_pb2.PublicAddress()
    public_address.ParseFromString(public_address_bytes)

    wrapper = printable_pb2.PrintableWrapper()
    wrapper.public_address.CopyFrom(public_address)

    wrapper_bytes = wrapper.SerializeToString()

    checksum = zlib.crc32(wrapper_bytes)
    checksum_bytes = checksum.to_bytes(4, byteorder="little")

    checksum_and_wrapper_bytes = checksum_bytes + wrapper_bytes

    return base58.b58encode(checksum_and_wrapper_bytes).decode("utf-8")


def b58_wrapper_to_b64_public_address(b58_string):
    """Convert a b58-encoded PrintableWrapper address into a b64-encoded PublicAddress protobuf"""
    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]

    wrapper = printable_pb2.PrintableWrapper()
    wrapper.ParseFromString(wrapper_bytes)
    public_address = wrapper.public_address

    public_address_bytes = public_address.SerializeToString()
    return base64.b64encode(public_address_bytes).decode("utf-8")


def b58_string_passes_checksum(b58_string):
    """Validate the checksum of a b58-encoded string"""
    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]
    checksum_bytes = checksum_and_wrapper_bytes[0:4]
    new_checksum = zlib.crc32(wrapper_bytes)
    new_checksum_bytes = new_checksum.to_bytes(4, byteorder="little")

    return checksum_bytes == new_checksum_bytes


def b58_string_is_public_address(b58_string):
    """Check if a b58-encoded string contains a PrintableWrapper protobuf with a PublicAddress"""
    if not b58_string_passes_checksum(b58_string):
        return False

    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]
    wrapper = printable_pb2.PrintableWrapper()

    try:
        wrapper.ParseFromString(wrapper_bytes)
        return wrapper.PublicAddress is not None
    except Exception:
        return False


def b64_receipt_to_full_service_receipt(b64_string):
    """Convert a b64-encoded protobuf Receipt into a full-service receipt object"""
    receipt_bytes = base64.b64decode(b64_string)
    receipt = external_pb2.Receipt.FromString(receipt_bytes)

    full_service_receipt = {
        "object": "receiver_receipt",
        "public_key": receipt.public_key.SerializeToString().hex(),
        "confirmation": receipt.confirmation.SerializeToString().hex(),
        "tombstone_block": str(int(receipt.tombstone_block)),
        "amount": {
            "object": "amount",
            "commitment": receipt.masked_amount_v2.commitment.data.hex(),
            "masked_value": str(int(receipt.masked_amount_v2.masked_value)),
            "masked_token_id": receipt.masked_amount_v2.masked_token_id.hex(),
            "version": "V2",
        },
    }

    return full_service_receipt


def full_service_receipt_to_b64_receipt(full_service_receipt):
    """Convert a full-service receipt object to a b64-encoded protobuf Receipt"""

    public_key = external_pb2.CompressedRistretto.FromString(
        bytes.fromhex(full_service_receipt["public_key"])
    )
    confirmation = external_pb2.TxOutConfirmationNumber.FromString(
        bytes.fromhex(full_service_receipt["confirmation"])
    )
    tombstone_block = int(full_service_receipt["tombstone_block"])
    amount_commitment = external_pb2.CompressedRistretto(
        data=bytes.fromhex(full_service_receipt["amount"]["commitment"])
    )
    amount_masked_value = int(full_service_receipt["amount"]["masked_value"])

    masked_amount = external_pb2.MaskedAmount(
        commitment=amount_commitment,
        masked_value=amount_masked_value,
        masked_token_id=bytes.fromhex(full_service_receipt["amount"]["masked_token_id"]),
    )
    r = external_pb2.Receipt(
        public_key=public_key,
        confirmation=confirmation,
        tombstone_block=tombstone_block,
        masked_amount_v2=masked_amount,
    )
    return base64.b64encode(r.SerializeToString()).decode("utf-8")

