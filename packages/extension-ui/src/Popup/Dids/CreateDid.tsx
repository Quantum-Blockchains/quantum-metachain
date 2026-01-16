// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { AccountContext, ActionContext, BackButton, ButtonArea, InputWithLabel, NextStepButton, ValidatedInput, VerticalSpace, Warning } from '../../components/index.js';
import BoxWithLabel from '../../components/BoxWithLabel.js';
import useToast from '../../hooks/useToast.js';
import useTranslation from '../../hooks/useTranslation.js';
import { Header } from '../../partials/index.js';
import { createDid } from '../../messaging.js';
import { allOf, isNotShorterThan, isSameAs } from '../../util/validators.js';
import AddressDropdown from '../Derive/AddressDropdown.js';
import { styled } from '../../styled.js';

interface Props extends ThemeProps {
  className?: string;
}

type Step = 'details' | 'sign';
const MIN_LENGTH = 6;

function CreateDid ({ className }: Props): React.ReactElement<Props> {
  const { t } = useTranslation();
  const { show } = useToast();
  const onAction = useContext(ActionContext);
  const { accounts } = useContext(AccountContext);
  const [isBusy, setIsBusy] = useState(false);
  const [name, setName] = useState('');
  const [didPass1, setDidPass1] = useState<string | null>(null);
  const [didPass2, setDidPass2] = useState<string | null>(null);
  const [didPassword, setDidPassword] = useState<string | null>(null);
  const [accountPassword, setAccountPassword] = useState('');
  const [generatedDid, setGeneratedDid] = useState<string | null>(null);
  const [selectedAddress, setSelectedAddress] = useState<string>('');
  const [step, setStep] = useState<Step>('details');
  const didPasswordValidator = useMemo(
    () => isNotShorterThan(MIN_LENGTH, t<string>('Password is too short')),
    [t]
  );
  const didPasswordRepeatValidator = useCallback(
    (firstPassword: string) => allOf(
      isNotShorterThan(MIN_LENGTH, t<string>('Password is too short')),
      isSameAs(firstPassword, t<string>('Passwords do not match'))
    ),
    [t]
  );

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

  useEffect(() => {
    setDidPassword(didPass1 && didPass2 ? didPass1 : null);
  }, [didPass1, didPass2]);

  const _onCreate = useCallback(async (name: string, accountPassword: string, didPassword: string): Promise<void> => {
    if (!name || !accountPassword || !didPassword || !selectedAddress) {
      if (!selectedAddress) {
        show(t<string>('Select an account to sign the DID creation.'));
      }

      return;
    }

    setIsBusy(true);
    try {
      const didRecord = await createDid(selectedAddress, name, accountPassword, didPassword);

      setGeneratedDid(didRecord.did);
      show(t<string>('DID created on chain.'));
      window.localStorage.setItem('accounts_active_tab', 'dids');
      window.location.hash = '/';
      onAction('/');
    } catch (error) {
      const message = error instanceof Error
        ? error.message
        : t<string>('DID creation failed.');

      show(message);
    } finally {
      setIsBusy(false);
    }
  }, [onAction, selectedAddress, show, t]);

  const _onContinue = useCallback((): void => {
    if (!name || !didPassword) {
      return;
    }

    setStep('sign');
  }, [didPassword, name]);

  const _onBack = useCallback((): void => {
    if (step === 'sign') {
      setStep('details');
    } else {
      onAction('../index.js');
    }
  }, [onAction, step]);

  return (
    <div className={className}>
      <Header
        showBackArrow
        showSettings
        text={t<string>('Generate DID')}
      />
      {generatedDid && (
        <div className='generatedDid'>
          <BoxWithLabel
            label={t<string>('Generated DID')}
            value={generatedDid}
          />
        </div>
      )}
      {step === 'details' && (
        <>
          <InputWithLabel
            label={t<string>('DID name')}
            onChange={setName}
            value={name}
          />
          <ValidatedInput
            component={InputWithLabel}
            data-input-did-password
            isFocused
            label={t<string>('DID password')}
            onValidatedChange={setDidPass1}
            type='password'
            validator={didPasswordValidator}
          />
          {didPass1 && (
            <ValidatedInput
              component={InputWithLabel}
              data-input-did-password-repeat
              label={t<string>('Repeat DID password')}
              onValidatedChange={setDidPass2}
              type='password'
              validator={didPasswordRepeatValidator(didPass1)}
            />
          )}
        </>
      )}
      {step === 'sign' && (
        <>
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
          <InputWithLabel
            label={t<string>('Password for selected account')}
            onChange={setAccountPassword}
            type='password'
            value={accountPassword}
          />
        </>
      )}
      <VerticalSpace />
      <ButtonArea>
        <BackButton onClick={_onBack} />
        {step === 'details' && (
          <NextStepButton
            data-button-action='continue-did-details'
            isDisabled={!name || !didPassword}
            onClick={_onContinue}
          >
            {t<string>('Continue')}
          </NextStepButton>
        )}
        {step === 'sign' && (
          <NextStepButton
            data-button-action='generate-did'
            isBusy={isBusy}
            isDisabled={!name || !accountPassword || !selectedAddress || !didPassword}
            onClick={() => didPassword && _onCreate(name, accountPassword, didPassword)}
          >
            {t<string>('Generate DID')}
          </NextStepButton>
        )}
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
