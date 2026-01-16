// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { RouteComponentProps } from 'react-router';
import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { withRouter } from 'react-router';

import { AccountContext, ActionBar, ActionContext, ActionText, Button, InputWithLabel, Warning } from '../../components/index.js';
import BoxWithLabel from '../../components/BoxWithLabel.js';
import useToast from '../../hooks/useToast.js';
import useTranslation from '../../hooks/useTranslation.js';
import { deactivateDid, didsList } from '../../messaging.js';
import { Header } from '../../partials/index.js';
import { styled } from '../../styled.js';
import AddressDropdown from '../Derive/AddressDropdown.js';

const MIN_LENGTH = 6;

interface Props extends RouteComponentProps<{ did: string }>, ThemeProps {
  className?: string;
}

function DeactivateDid ({ className, match: { params: { did } } }: Props): React.ReactElement<Props> {
  const { t } = useTranslation();
  const { show } = useToast();
  const onAction = useContext(ActionContext);
  const { accounts } = useContext(AccountContext);
  const [isBusy, setIsBusy] = useState(false);
  const [accountPassword, setAccountPassword] = useState('');
  const [didPassword, setDidPassword] = useState('');
  const [error, setError] = useState('');
  const [accountAddress, setAccountAddress] = useState<string>('');
  const normalizedDid = useMemo(() => decodeURIComponent(did), [did]);
  const accountOptions = useMemo(
    () => accounts
      .filter(({ isExternal, isHardware }) => !isExternal && !isHardware)
      .map(({ address, genesisHash }) => [address, genesisHash || null] as [string, string | null]),
    [accounts]
  );
  const hasAccounts = accountOptions.length > 0;

  useEffect(() => {
    didsList()
      .then((items) => {
        const record = items.find((item) => item.did === normalizedDid);

        if (record?.accountAddress) {
          setAccountAddress(record.accountAddress);
        }
      })
      .catch(console.error);
  }, [normalizedDid]);

  useEffect(() => {
    if (!accountAddress && hasAccounts) {
      setAccountAddress(accountOptions[0][0]);
    }
  }, [accountAddress, accountOptions, hasAccounts]);

  const _goHome = useCallback(
    () => onAction('/'),
    [onAction]
  );

  const onAccountPassChange = useCallback(
    (password: string) => {
      setAccountPassword(password);
      setError('');
    },
    []
  );

  const onDidPassChange = useCallback(
    (password: string) => {
      setDidPassword(password);
      setError('');
    },
    []
  );

  const _onDeactivateClick = useCallback(
    (): void => {
      setIsBusy(true);

      deactivateDid(normalizedDid, accountAddress, accountPassword, didPassword)
        .then(() => {
          show(t<string>('DID deactivated on-chain.'));
          window.localStorage.setItem('accounts_active_tab', 'dids');
          onAction('/');
        })
        .catch((error: Error) => {
          console.error(error);
          setError(error.message);
          setIsBusy(false);
        });
    },
    [accountAddress, accountPassword, didPassword, normalizedDid, onAction, show, t]
  );

  return (
    <>
      <Header
        showBackArrow
        text={t<string>('Deactivate DID')}
      />
      <div className={className}>
        <BoxWithLabel
          label={t<string>('DID')}
          value={normalizedDid}
        />
        <Warning className='movedWarning'>
          {t<string>('Deactivating a DID is permanent and cannot be undone.')}
        </Warning>
        <div className='actionArea'>
          {hasAccounts && accountAddress && (
            <AddressDropdown
              allAddresses={accountOptions}
              onSelect={setAccountAddress}
              selectedAddress={accountAddress}
              selectedGenesis={accountOptions.find(([address]) => address === accountAddress)?.[1] || null}
            />
          )}
          <InputWithLabel
            data-deactivate-password
            disabled={isBusy}
            isError={accountPassword.length < MIN_LENGTH || !!error}
            label={t<string>('password for the signing account')}
            onChange={onAccountPassChange}
            type='password'
          />
          <InputWithLabel
            data-did-password
            disabled={isBusy}
            isError={didPassword.length < MIN_LENGTH || !!error}
            label={t<string>('password for this DID')}
            onChange={onDidPassChange}
            type='password'
          />
          {error && (
            <Warning
              isBelowInput
              isDanger
            >
              {error}
            </Warning>
          )}
          <Button
            className='deactivate-button'
            data-deactivate-button
            isBusy={isBusy}
            isDanger
            isDisabled={!accountAddress || accountPassword.length === 0 || didPassword.length === 0 || !!error}
            onClick={_onDeactivateClick}
          >
            {t<string>('Deactivate DID')}
          </Button>
          <ActionBar className='withMarginTop'>
            <ActionText
              className='center'
              onClick={_goHome}
              text={t<string>('Cancel')}
            />
          </ActionBar>
        </div>
      </div>
    </>
  );
}

export default withRouter(styled(DeactivateDid)`
  .actionArea {
    padding: 10px 24px;
  }

  .center {
    margin: auto;
  }

  .deactivate-button {
    margin-top: 6px;
  }

  .movedWarning {
    margin-top: 8px;
  }

  .withMarginTop {
    margin-top: 4px;
  }
`);
