import onetimepad


def encrypt(plaintext, key):
    """
    Encrypts the plaintext using the specified key.
    """
    return onetimepad.encrypt(plaintext, key)


def decrypt(ciphertext, key):
    """
    Decrypts the ciphertext using the specified key.
    """
    return onetimepad.decrypt(ciphertext, key)
