from decimal import Decimal
import zlib
import base58
import base64
import external_pb2
import printable_pb2


PMOB = Decimal('1e12')

def mob2pmob(x):
    """ Convert from MOB to picoMOB. """
    return round(Decimal(x) * PMOB)


def pmob2mob(x):
    """ Convert from picoMOB to MOB. """
    result = int(x) / PMOB
    if result == 0:
        return Decimal('0')
    else:
        return result


def try_int(x):
    if x is not None:
        return int(x)

def b64_public_address_to_b58_wrapper(b64_string):
    public_address_bytes = base64.b64decode(b64_string)

    public_address = external_pb2.PublicAddress()
    public_address.ParseFromString(public_address_bytes)

    wrapper = printable_pb2.PrintableWrapper()
    wrapper.public_address.CopyFrom(public_address)

    wrapper_bytes = wrapper.SerializeToString()

    checksum = zlib.crc32(wrapper_bytes)
    checksum_bytes = checksum.to_bytes(4, byteorder="little")

    checksum_and_wrapper_bytes = checksum_bytes + wrapper_bytes

    return base58.b58encode(checksum_and_wrapper_bytes).decode('utf-8')

def b58_wrapper_to_b64_public_address(b58_string):
    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]

    wrapper = printable_pb2.PrintableWrapper()
    wrapper.ParseFromString(wrapper_bytes)
    public_address = wrapper.public_address

    public_address_bytes = public_address.SerializeToString()
    return base64.b64encode(public_address_bytes).decode('utf-8')

def b58_string_passes_checksum(b58_string):
    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]
    checksum_bytes = checksum_and_wrapper_bytes[0:4]
    new_checksum = zlib.crc32(wrapper_bytes)
    new_checksum_bytes = new_checksum.to_bytes(4, byteorder="little")

    return checksum_bytes == new_checksum_bytes

def b58_string_is_public_address(b58_string):
    if not b58_string_passes_checksum(b58_string):
        return False
    
    checksum_and_wrapper_bytes = base58.b58decode(b58_string)
    wrapper_bytes = checksum_and_wrapper_bytes[4:]
    wrapper = printable_pb2.PrintableWrapper()

    try:
        wrapper.ParseFromString(wrapper_bytes)
        return wrapper.PublicAddress is not None
    except:
        return False