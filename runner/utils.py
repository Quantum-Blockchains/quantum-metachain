import base58
import base64


def base58_to_hex(val: str):
    hex = base58.b58decode(val).hex()
    # hex[:12] is the Network ID
    return f"{hex[12:]}"


def base64_to_hex(message: str):
    return f"0x{base64.b64decode(message).hex()}"


def xor(s1: str, s2: str):
    """
    xor_two_str accepts two strings as input, converts them to bytes and perform XOR operation.
    """
    result = int(s1, base=16) ^ int(s2, base=16)
    return hex(result)
