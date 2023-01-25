import base64

import base58


def base58_to_hex(val: str):
    return base58.b58decode(val).hex()


def base64_to_hex(message: str):
    return f"0x{base64.b64decode(message).hex()}"


def xor(s1: str, s2: str):
    """
    xor_two_str accepts two strings as input, converts them to bytes and perform XOR operation.
    """
    result = int(s1, base=16) ^ int(s2, base=16)
    return hex(result)


def trim_0x_prefix(val: str):
    return format(int(val, 16), 'x')
