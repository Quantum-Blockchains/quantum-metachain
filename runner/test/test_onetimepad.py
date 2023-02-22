from core.onetimepad import encrypt, decrypt


def test_encrypt_decrypt_correct_lengths():
    psk = "f49d9ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "b54232ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk


def test_encrypt_decrypt_():
    psk = "00009ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "b54232ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk


def test_encrypt_decrypt_successful():
    psk = "00009ac6f08e665149ee62b1de8230e996f5bb68c37d54b16810d97e354aed7c"
    qkd_key = "0000032ccdd252daedadcd4a470699193343b5b9b600ab1c2dee523d101db62ac"

    encrypted = encrypt(psk, qkd_key)
    decrypted = decrypt(encrypted, qkd_key)
    assert decrypted == psk
