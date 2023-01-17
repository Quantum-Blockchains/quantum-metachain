from crypto import sign, verify, to_public, to_public_from_peerid


def test_public_key_conversion():
    peer_id = "12D3KooWKzWKFojk7A1Hw23dpiQRbLs6HrXFf4EGLsN4oZ1WsWCc"
    private_key = "df432c8e967aa21fdd287d3ea61fa85640a8309577f65b4ea78d49d514661654"

    assert to_public(private_key) == to_public_from_peerid(peer_id)

def test_signature_flow_successful():
    private_key = "168bcc3789d741afc6e3f422f03da05fd4877e3e5518681758043ed7734967e9"
    data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"
    public_key = to_public(private_key)

    signature = sign(data, private_key)

    assert verify(data, signature, public_key)


def test_signature_flow_unsuccessful_when_pubkey_is_different():
    alice_private_key = "168bcc3789d741afc6e3f422f03da05fd4877e3e5518681758043ed7734967e9"
    bob_private_key = "0000000000000000000000000000000000000000000000000000000000000000"
    bob_public_key = to_public(bob_private_key)
    data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"

    signature = sign(data, alice_private_key)

    assert not verify(data, signature, bob_public_key)


def test_verification_flow_successful():
    data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"
    public_key = bytes.fromhex("da94c76735530f88f286dabc3785e69d82920ecfbedae3ab068c5df58709644e")
    signature = bytes.fromhex("fae47d3f21430743df8062d9c4a82cee5df7606d5672413d13aea657eb248d3f917f7487b7e154437515162903396bf3c827d54ea9ff2c9bf47290804d96630b")

    assert verify(data, signature, public_key)


def test_verification_flow_successful_alice():
    data = "4fee5778d20391de7560fb5d13b1f2771be64ea3593a9fff4ef50412fe67e077"
    private_key = "ccaa338f88c037721be481400e59b255431fff0bc4440807085c84bb4848d786"
    public_key = to_public(private_key)
    signature = bytes.fromhex("4dbc9a0e938ed12db36e115248ad713e1bad420a2c6d17bf091b367e2fcba2e9e410ccbef051f5607b776325a88aa1bc2ba13b8fdd3484ca5fae47fd18fd4e01")
    test_signature = sign(data, private_key)

    assert verify(data, signature, public_key)
    assert test_signature == signature


def test_verification_flow_unsuccessful_when_pubkey_is_different():
    data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"
    public_key = bytes.fromhex("0000000000000000000000000000000000000000000000000000000000000000")
    signature = bytes.fromhex("fae47d3f21430743df8062d9c4a82cee5df7606d5672413d13aea657eb248d3f917f7487b7e154437515162903396bf3c827d54ea9ff2c9bf47290804d96630b")

    assert not verify(data, signature, public_key)


def test_verification_flow_unsuccessful_when_signature_is_different():
    data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"
    public_key = bytes.fromhex("da94c76735530f88f286dabc3785e69d82920ecfbedae3ab068c5df58709644e")
    signature = bytes.fromhex("abcded3f21430743af8062d9c4a82cee5df7606d5672413d13aea657eb248d3f917f7487b7e154437515162903396bf3c827d54ea9ff2c9bf47290804d96630b")

    assert not verify(data, signature, public_key)
