from dataclasses import dataclass

import requests
import validators

from common.logger import log


@dataclass(frozen=True)
class QNULabsProvider:
    config: dict

    def get_enc_key(self):
        qkd_url = f"{self.config['url']}/enc_keys?size=64"
        if not validators.url(qkd_url):
            raise requests.exceptions.InvalidURL

        client_cert_path = self.config.get("client_cert_path")
        cert_key_path = self.config.get("cert_key_path")
        response = requests.get(qkd_url, cert=(client_cert_path, cert_key_path), verify=False).json()
        log.debug(f"response from qkd: {response}")

        key = response["keys"][0]
        return key["key_ID"], key["key"]

    def get_dec_key(self, key_id):
        qkd_url = f"{self.config['url']}/dec_keys?key_ID={key_id}"
        if not validators.url(qkd_url):
            raise requests.exceptions.InvalidURL

        client_cert_path = self.config.get("client_cert_path")
        cert_key_path = self.config.get("cert_key_path")
        response = requests.get(qkd_url, cert=(client_cert_path, cert_key_path), verify=False).json()
        log.debug(f"response from qkd: {response}")

        return response["key_ID"], response["key"]
