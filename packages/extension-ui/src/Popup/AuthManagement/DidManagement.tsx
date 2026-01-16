// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { DidRecord } from '@polkadot/extension-base/background/types';
import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';
import { useParams } from 'react-router';

import { ActionContext, Button, Checkbox, Warning } from '../../components/index.js';
import useTranslation from '../../hooks/useTranslation.js';
import { didsList, getAuthList, updateAuthorization } from '../../messaging.js';
import { Header } from '../../partials/index.js';
import { styled } from '../../styled.js';

interface Props extends ThemeProps {
  className?: string;
}

function DidManagement ({ className }: Props): React.ReactElement<Props> {
  const { url } = useParams<{url: string}>();
  const { t } = useTranslation();
  const onAction = useContext(ActionContext);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const [selectedDids, setSelectedDids] = useState<string[]>([]);
  const [authorizedAccounts, setAuthorizedAccounts] = useState<string[]>([]);
  const [savedAuthorizedDids, setSavedAuthorizedDids] = useState<string[] | undefined>(undefined);
  const didIds = useMemo(() => dids.map((item) => item.did), [dids]);
  const noDidSelected = useMemo(() => selectedDids.length === 0, [selectedDids.length]);
  const areAllDidsSelected = useMemo(
    () => selectedDids.length === didIds.length,
    [didIds.length, selectedDids.length]
  );
  const [isIndeterminate, setIsIndeterminate] = useState(false);

  useEffect(() => {
    getAuthList()
      .then(({ list }) => {
        if (!list[url]) {
          return;
        }

        setAuthorizedAccounts(list[url].authorizedAccounts || []);
        setSavedAuthorizedDids(list[url].authorizedDids);
      })
      .catch(console.error);
  }, [url]);

  useEffect(() => {
    didsList()
      .then(setDids)
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (!dids.length) {
      return;
    }

    if (savedAuthorizedDids === undefined) {
      setSelectedDids(dids.map((item) => item.did));
    } else {
      setSelectedDids(savedAuthorizedDids.filter((did) => didIds.includes(did)));
    }
  }, [dids, didIds, savedAuthorizedDids]);

  useEffect(() => {
    const nextIndeterminateState = !noDidSelected && !areAllDidsSelected;

    setIsIndeterminate(nextIndeterminateState);
  }, [areAllDidsSelected, noDidSelected]);

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

  const _onApprove = useCallback(
    (): void => {
      updateAuthorization(authorizedAccounts, url, selectedDids)
        .then(() => onAction('../index.js'))
        .catch(console.error);
    },
    [authorizedAccounts, onAction, selectedDids, url]
  );

  return (
    <>
      <Header
        showBackArrow
        smallMargin={true}
        text={t<string>('DIDs connected to {{url}}', { replace: { url } })}
      />
      <div className={className}>
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
                  <div className='didItem' key={did}>
                    <div className='didRow'>
                      <div className='didIcon' aria-hidden='true'>
                        <span>ID</span>
                      </div>
                      <div className='didBody'>
                        <div className='didName'>{name || t<string>('DID')}</div>
                        <div className='didValue'>{did}</div>
                      </div>
                      <div className='didCheckbox'>
                        <Checkbox
                          checked={selectedDids.includes(did)}
                          className='did-checkbox'
                          label=''
                          onChange={() => _onToggleDid(did)}
                        />
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        <Button
          className='acceptButton'
          onClick={_onApprove}
        >
          {t<string>('Connect {{total}} DID(s)', { replace: {
            total: selectedDids.length
          } })}
        </Button>
      </div>
    </>
  );
}

export default styled(DidManagement)(({ theme }) => `
  .didList {
    height: 360px;
    overflow-y: auto;
    margin-top: 6px;
  }

  .didItem {
    border: 1px solid ${theme.inputBorderColor};
    border-radius: 8px;
    background: ${theme.readonlyInputBackground};
    padding: 10px 12px;
    margin-bottom: 10px;
  }

  .didRow {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .didBody {
    min-width: 0;
    flex: 1;
  }

  .didName {
    color: ${theme.textColor};
    font-size: 14px;
    line-height: 18px;
  }

  .didValue {
    color: ${theme.labelColor};
    font-size: 12px;
    line-height: 16px;
    word-break: break-all;
  }

  .didIcon {
    width: 32px;
    height: 32px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #2e7d6a;
    color: #fff;
    font-size: 14px;
    flex: 0 0 32px;
  }

  .didCheckbox {
    display: inline-flex;
    align-items: center;
    align-self: center;
  }

  .didCheckbox .checkbox {
    display: flex;
    align-items: center;
    margin: 0;
  }

  .didWarning {
    margin: 6px 0;
  }

  .acceptButton {
    width: 90%;
    margin: 0.5rem auto 0;
  }
`);
