// Copyright 2019-2023 @polkadot/extension-base authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { KeyringPair$Json } from '@polkadot/keyring/types';

import { EXTENSION_PREFIX } from '../defaults.js';
import BaseStore from './Base.js';

export default class DidsStore extends BaseStore<KeyringPair$Json> {
  constructor () {
    super(
      EXTENSION_PREFIX && EXTENSION_PREFIX !== 'polkadot{.js}'
        ? `${EXTENSION_PREFIX}dids`
        : 'dids'
    );
  }
}
