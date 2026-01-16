// Copyright 2019-2023 @polkadot/extension-base authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { DidRecord, RequestDidSign, ResponseDidSign } from '../background/types.js';
import type { SendRequest } from './types.js';

export default class Dids {
  readonly #sendRequest: SendRequest;

  constructor (sendRequest: SendRequest) {
    this.#sendRequest = sendRequest;
  }

  public list (): Promise<DidRecord[]> {
    return this.#sendRequest('pub(dids.list)');
  }

  public sign (request: RequestDidSign): Promise<ResponseDidSign> {
    return this.#sendRequest('pub(dids.sign)', request);
  }
}
