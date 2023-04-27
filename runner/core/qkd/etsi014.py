from dataclasses import dataclass

import requests
import validators

from common import crypto
from common.logger import log


@dataclass(frozen=True)
class ETSI014Provider:

    config: dict

    def get_enc_key(self):
        qkd_url = f"{self.config['url']}/enc_keys?size=256"
        if not validators.url(qkd_url):
            raise requests.exceptions.InvalidURL

        response = self._call_qkd(qkd_url, self.config.get("client_cert_path"), self.config.get("cert_key_path"))
        log.debug(f"response from qkd: {response}")
        return self.__unwrap_response(response)

    def get_dec_key(self, key_id):
        qkd_url = f"{self.config['url']}/dec_keys?key_ID={key_id}"
        if not validators.url(qkd_url):
            raise requests.exceptions.InvalidURL

        response = self._call_qkd(qkd_url, self.config.get("client_cert_path"), self.config.get("cert_key_path"))
        log.debug(f"response from qkd: {response}")
        return self.__unwrap_response(response)

    @staticmethod
    def _call_qkd(qkd_url, cert_path=None, key_path=None):
        if cert_path is None or key_path is None:
            return requests.get(qkd_url).json()
        else:
            return requests.get(qkd_url, cert=(cert_path, key_path), verify=False).json()

    @staticmethod
    def __unwrap_response(response):
        key = response["keys"][0]
        key_id = key["key_ID"]
        qkd_key = key["key"]
        decoded_qkd_key = crypto.base64_to_hex(qkd_key)

        return key_id, decoded_qkd_key
