import pytest

from core.onetimepad import encrypt, decrypt


def test_encrypt_decrypt_correct_lengths():
    psk = "f49d9ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "b54232ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk


def test_encrypt_decrypt_when_psk_with_leading_zeros():
    psk = "00009ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "b54232ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk


def test_encrypt_decrypt_successful_when_psk_and_qkd_key_with_leading_zeros():
    psk = "00009ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "000032ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk


def test_invalid_key_length():
    text = "hello"
    key = "2a6c1a1a"
    with pytest.raises(ValueError, match="Text and key must have the same length."):
        encrypt(text, key)


def test_non_hex_key():
    text = "hello"
    key = "a6c1g"
    with pytest.raises(ValueError, match="Key must be a hexadecimal string."):
        encrypt(text, key)
