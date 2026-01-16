// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { DidRecord, RequestAuthorizeTab } from '@polkadot/extension-base/background/types';
import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import { AccountContext, ActionBar, ActionContext, Button, Checkbox, Link, Warning } from '../../components/index.js';
import useTranslation from '../../hooks/useTranslation.js';
import { approveAuthRequest, deleteAuthRequest, didsList } from '../../messaging.js';
import { AccountSelection } from '../../partials/index.js';
import { styled } from '../../styled.js';
import NoAccount from './NoAccount.js';

interface Props extends ThemeProps {
  authId: string;
  className?: string;
  isFirst: boolean;
  request: RequestAuthorizeTab;
  url: string;
}

function Request ({ authId, className, isFirst, request: { origin }, url }: Props): React.ReactElement<Props> {
  const { accounts, selectedAccounts = [], setSelectedAccounts } = useContext(AccountContext);
  const { t } = useTranslation();
  const onAction = useContext(ActionContext);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const [selectedDids, setSelectedDids] = useState<string[]>([]);
  const [isIndeterminate, setIsIndeterminate] = useState(false);

  useEffect(() => {
    const defaultAccountSelection = accounts
      .filter(({ isDefaultAuthSelected }) => !!isDefaultAuthSelected)
      .map(({ address }) => address);

    setSelectedAccounts && setSelectedAccounts(defaultAccountSelection);
  }, [accounts, setSelectedAccounts]);

  useEffect(() => {
    didsList()
      .then((items) => {
        setDids(items);
        setSelectedDids(items.map((item) => item.did));
      })
      .catch(console.error);
  }, []);

  const didIds = useMemo(() => dids.map((item) => item.did), [dids]);
  const noDidSelected = useMemo(() => selectedDids.length === 0, [selectedDids.length]);
  const areAllDidsSelected = useMemo(
    () => selectedDids.length === didIds.length,
    [didIds.length, selectedDids.length]
  );

  useEffect(() => {
    const nextIndeterminateState = !noDidSelected && !areAllDidsSelected;

    setIsIndeterminate(nextIndeterminateState);
  }, [areAllDidsSelected, noDidSelected]);

  const _onApprove = useCallback(
    (): void => {
      approveAuthRequest(authId, selectedAccounts, selectedDids)
        .then(() => onAction())
        .catch((error: Error) => console.error(error));
    },
    [authId, onAction, selectedAccounts, selectedDids]
  );

  const _onClose = useCallback(
    (): void => {
      deleteAuthRequest(authId)
        .then(() => onAction())
        .catch((error: Error) => console.error(error));
    },
    [authId, onAction]
  );

  if (!accounts.length) {
    return <NoAccount authId={authId} />;
  }

  const _onSelectAllDids = (): void => {
    if (areAllDidsSelected) {
      setSelectedDids([]);
      return;
    }

    setSelectedDids(didIds);
  };

  const _onToggleDid = (did: string): void => {
    setSelectedDids((current) =>
      current.includes(did)
        ? current.filter((item) => item !== did)
        : [...current, did]
    );
  };

  return (
    <div className={className}>
      <AccountSelection
        origin={origin}
        url={url}
      />
      <div className='didSelection'>
        <div className='didHeader'>{t<string>('Select DIDs to share')}</div>
        {dids.length === 0
          ? (
            <Warning className='didWarning'>
              {t<string>('No DIDs found in your wallet.')}
            </Warning>
          )
          : (
            <>
              <Checkbox
                checked={areAllDidsSelected}
                className='did-checkbox'
                indeterminate={isIndeterminate}
                label={t<string>('Select all DIDs')}
                onChange={_onSelectAllDids}
              />
              <div className='didList'>
                {dids.map(({ did, name }) => (
                  <Checkbox
                    checked={selectedDids.includes(did)}
                    className='did-checkbox'
                    key={did}
                    label={name ? `${name} (${did})` : did}
                    onChange={() => _onToggleDid(did)}
                  />
                ))}
              </div>
            </>
          )}
      </div>
      {isFirst && (
        <Button
          className='acceptButton'
          onClick={_onApprove}
        >
          {t<string>('Connect {{total}} account(s)', { replace: {
            total: selectedAccounts.length
          } })}
        </Button>
      )}
      <ActionBar className='rejectionButton'>
        <Link
          className='closeLink'
          isDanger
          onClick={_onClose}
        >
          {t<string>('Ask again later')}
        </Link>
      </ActionBar>
    </div>
  );
}

export default styled(Request)(({ theme }: ThemeProps) => `
  .didSelection {
    margin-top: 12px;
    padding-bottom: 6px;
  }

  .didHeader {
    color: ${theme.textColor};
    font-size: 14px;
    margin: 6px 0;
  }

  .didList {
    max-height: 140px;
    overflow-y: auto;
    padding: 6px 0;
  }

  .didWarning {
    margin: 6px 0 0;
  }

  .acceptButton {
    width: 90%;
    margin: 12px auto 0;
  }

  .rejectionButton {
    margin: 0 0 15px 0;
    text-decoration: underline;

    .closeLink {
      margin: auto;
      padding: 0;
    }
  }
`);
