// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { HexString } from '@polkadot/util/types';

import { useMemo } from 'react';

interface Option {
  text: string;
  value: HexString;
}

export default function (): Option[] {
  const hashes = useMemo(() => [
    {
      text: 'QSB-Poseidon',
      value: '0x70ed91ab00658641696a9b29f23770fd9112e6a677318734521e5dd8f7ac72d2' as HexString
    }
  ], []);

  return hashes;
}
