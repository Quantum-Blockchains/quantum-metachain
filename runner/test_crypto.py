from crypto import sign, verify, to_public


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
