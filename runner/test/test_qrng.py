from psk import get_psk
import config


config.config_service = config.ConfigService(config.Config())
url = f"https://qrng.qbck.io/{config.config_service.current_config.qrng_api_key}/qbck/block/hex?size=1&length=32"


def test_get_psk_from_qrng(requests_mock):
    expected_psk = "0xa9d6e6fd9b9fbdd2527b2b7919d0e19e2c5b64e9cb554760d8aa686c0131f282"
    qrng_response = {
        "data": {
            "result": [
                expected_psk
            ],
        }
    }
    requests_mock.get(url, json=qrng_response)

    result = get_psk()

    assert result == expected_psk


def test_get_psk_from_random(requests_mock):
    requests_mock.get(url, json={}, status_code=500)

    result = get_psk()

    assert len(result) == 64
    assert int(result, 16)
