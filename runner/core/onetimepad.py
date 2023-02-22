def xor_bytes(a, b):
    """Performs a bitwise XOR of two strings. Returns a byte string."""
    return bytes([ord(x) ^ ord(y) for x, y in zip(a, b)])


def encrypt(plaintext, key):
    """
    Encrypts the plaintext using the specified key.
    """
    bytetext = xor_bytes(plaintext, key)
    return bytetext.decode("utf-8")


def decrypt(ciphertext, key):
    """
    Decrypts the ciphertext using the specified key.
    Please note that encrypt returns byte string, but encrypting function should only accept string as an argument
    (because of ord() in xor_bytes()).
    """
    bytetext = xor_bytes(ciphertext, key)
    return bytetext.decode("utf-8")
