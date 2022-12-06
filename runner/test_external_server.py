from external_server import xor_two_str


def test_xor_two_str():
    assert xor_two_str("0x123", "0x123") == "0x123"
