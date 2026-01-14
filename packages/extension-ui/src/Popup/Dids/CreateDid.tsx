// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { AccountContext, ActionContext, BackButton, ButtonArea, InputWithLabel, NextStepButton, VerticalSpace, Warning } from '../../components/index.js';
import BoxWithLabel from '../../components/BoxWithLabel.js';
import useToast from '../../hooks/useToast.js';
import useTranslation from '../../hooks/useTranslation.js';
import { Header } from '../../partials/index.js';
import { createDid } from '../../messaging.js';
import AddressDropdown from '../Derive/AddressDropdown.js';
import { styled } from '../../styled.js';

interface Props extends ThemeProps {
  className?: string;
}

function CreateDid ({ className }: Props): React.ReactElement<Props> {
  const { t } = useTranslation();
  const { show } = useToast();
  const onAction = useContext(ActionContext);
  const { accounts } = useContext(AccountContext);
  const [isBusy, setIsBusy] = useState(false);
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [generatedDid, setGeneratedDid] = useState<string | null>(null);
  const [selectedAddress, setSelectedAddress] = useState<string>('');

  const accountOptions = useMemo(
    () => accounts
      .filter(({ isExternal, isHardware }) => !isExternal && !isHardware)
      .map(({ address, genesisHash }) => [address, genesisHash || null] as [string, string | null]),
    [accounts]
  );
  const hasAccounts = accountOptions.length > 0;

  useEffect(() => {
    if (!selectedAddress && hasAccounts) {
      setSelectedAddress(accountOptions[0][0]);
    }
  }, [accountOptions, hasAccounts, selectedAddress]);

  const _onCreate = useCallback(async (name: string, password: string): Promise<void> => {
    if (!name || !password || !selectedAddress) {
      if (!selectedAddress) {
        show(t<string>('Select an account to sign the DID creation.'));
      }

      return;
    }

    setIsBusy(true);
    try {
      const didRecord = await createDid(selectedAddress, name, password);

      setGeneratedDid(didRecord.did);
      show(t<string>('DID created on chain.'));
    } catch (error) {
      const message = error instanceof Error
        ? error.message
        : t<string>('DID creation failed.');

      show(message);
    } finally {
      setIsBusy(false);
    }
  }, [selectedAddress, show, t]);

  return (
    <div className={className}>
      <Header
        showBackArrow
        showSettings
        text={t<string>('Generate DID')}
      />
      {hasAccounts && selectedAddress && (
        <AddressDropdown
          allAddresses={accountOptions}
          onSelect={setSelectedAddress}
          selectedAddress={selectedAddress}
          selectedGenesis={accountOptions.find(([address]) => address === selectedAddress)?.[1] || null}
        />
      )}
      {!hasAccounts && (
        <Warning className='noAccounts'>
          {t<string>('No accounts available. Add an account to sign the DID creation.')}
        </Warning>
      )}
      {generatedDid && (
        <div className='generatedDid'>
          <BoxWithLabel
            label={t<string>('Generated DID')}
            value={generatedDid}
          />
        </div>
      )}
      <InputWithLabel
        label={t<string>('DID name')}
        onChange={setName}
        value={name}
      />
      <InputWithLabel
        label={t<string>('Password for selected account')}
        onChange={setPassword}
        type='password'
        value={password}
      />
      <VerticalSpace />
      <ButtonArea>
        <BackButton onClick={() => onAction('../index.js')} />
        <NextStepButton
          data-button-action='generate-did'
          isBusy={isBusy}
          isDisabled={!name || !password || !selectedAddress}
          onClick={() => _onCreate(name, password)}
        >
          {t<string>('Generate DID')}
        </NextStepButton>
      </ButtonArea>
    </div>
  );
}

export default styled(CreateDid)(({ theme }: Props) => `
  color: ${theme.textColor};
  height: 100%;

  .generatedDid {
    margin-bottom: 12px;
  }

  .generatedDid .seedBox {
    word-break: break-all;
  }

  & ${VerticalSpace} {
    height: 16px;
  }
`);
