import onetimepad

from common.crypto import is_hex


def encrypt(plaintext, key):
    """
    Encrypts the plaintext using the specified key.
    """
    __validate_input(plaintext, key)
    return onetimepad.encrypt(plaintext, key)


def decrypt(ciphertext, key):
    """
    Decrypts the ciphertext using the specified key.
    """
    return onetimepad.decrypt(ciphertext, key)


def __validate_input(text, key):
    """
    Validates input for encryption and decryption.
    Raises an exception if the input is not valid.
    """
    if len(text) != len(key):
        raise ValueError(f"Text and key must have the same length., text {len(text)}, key {len(key)}")

    if not is_hex(key):
        raise ValueError("Key must be a hexadecimal string.")
