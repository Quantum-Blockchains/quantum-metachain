// Copyright 2019-2023 @polkadot/extension-base authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { KeyringPair } from '@polkadot/keyring/types';
import type { TypeRegistry } from '@polkadot/types';
import type { SignerPayloadJSON, SignerPayloadRaw } from '@polkadot/types/types';
import type { HexString } from '@polkadot/util/types';
import type { RequestSign } from './types.js';

import { hexToU8a, isHex, u8aToHex } from '@polkadot/util';
import { mldsa44Verify } from '@polkadot/util-crypto';

function isJsonPayload (value: SignerPayloadJSON | SignerPayloadRaw): value is SignerPayloadJSON {
  return (value as SignerPayloadJSON).genesisHash !== undefined;
}

function extractPayloadHex (value: SignerPayloadJSON | SignerPayloadRaw): HexString {
  const payload = isJsonPayload(value)
    ? value.method
    : value.data;

  if (!isHex(payload)) {
    throw new Error('Invalid DID sign payload. Expected hex in payload.method or payload.data');
  }

  return payload;
}

export default class RequestDidPayloadSign implements RequestSign {
  public readonly payload: SignerPayloadJSON | SignerPayloadRaw;

  constructor (payload: SignerPayloadJSON | SignerPayloadRaw | HexString) {
    if (typeof payload === 'string') {
      if (!isHex(payload)) {
        throw new Error('Invalid DID sign payload. Expected a 0x-prefixed hex string');
      }

      this.payload = {
        address: '',
        data: payload,
        type: 'bytes'
      };
    } else {
      this.payload = payload;
    }
  }

  sign (_registry: TypeRegistry, pair: KeyringPair): { signature: HexString } {
    const payloadU8a = hexToU8a(extractPayloadHex(this.payload));
    const signature = pair.sign(payloadU8a);
    const signatureRaw = signature.length === 2421
      ? signature.subarray(1)
      : signature;
    const isValid = mldsa44Verify(payloadU8a, signatureRaw, pair.publicKey);

    if (!isValid) {
      throw new Error('Invalid DID signature generated locally');
    }

    return {
      signature: u8aToHex(signatureRaw)
    };
  }
}
