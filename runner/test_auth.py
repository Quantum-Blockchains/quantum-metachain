from auth import verify_signature_agata, sign
from config import abs_node_key_file_path


def test_signature_flow_success():
    sig_path = "test_files/psk_signature"
    # we need a testing private key bytes for tests
    key_path = abs_node_key_file_path()
    psk_data = "18617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"

    sig = sign(psk_data, key_path, sig_path)

    try:
        with open(sig_path, "r") as file:
            sig_from_file = file.read()
    except Exception as e:
        print(e)

    assert verify_signature_agata(psk_data, sig)
    assert sig_from_file == sig


# def test_signature_flow_wrong_data():
#     sig_path = "test_files/psk_signature"
#     # we need a testing private key bytes for tests
#     key_path = "test_files/private_key"
#     psk_data = "28617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"

#     sign(psk_data, key_path, sig_path)

#     assert not verify_signature("some_random_data", sig_path, key_path)


# def test_signature_flow_wrong_key():
#     sig_path = "test_files/psk_signature"
#     # we need a testing private key bytes for tests
#     key_path = "test_files/private_key"
#     psk_data = "28617dff4efef20450dd5eafc060fd85faacca13d95ace3bda0be32e4694fcd7"

#     sign(psk_data, key_path, sig_path)

#     assert not verify_signature(psk_data, sig_path, "test_files/wrong_private_key")
