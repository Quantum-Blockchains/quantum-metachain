from utils import xor, base64_to_hex


def test_decode_base64():
    result = base64_to_hex("qV4XorklC1EbehIbsovSaRGlWhyw3jETpt/laDSr3BQ=")
    assert result == "0xa95e17a2b9250b511b7a121bb28bd26911a55a1cb0de3113a6dfe56834abdc14"


def test_xor():
    qkd_key = "0xa95e17a2b9250b511b7a121bb28bd26911a55a1cb0de3113a6dfe56834abdc14"
    psk = "0xa9d6e6fd9b9fbdd2527b2b7919d0e19e2c5b64e9cb554760d8aa686c0131f282"

    encrypted_result = xor(qkd_key, psk)

    assert encrypted_result == "0x88f15f22bab68349013962ab5b33f73dfe3ef57b8b76737e758d04359a2e96"
    assert xor(qkd_key, encrypted_result) == psk
