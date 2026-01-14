// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { AccountWithChildren, DidRecord } from '@polkadot/extension-base/background/types';
import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useMemo, useState } from 'react';

import getNetworkMap from '@polkadot/extension-ui/util/getNetworkMap';

import { AccountContext, ActionContext, Button } from '../../components/index.js';
import useTranslation from '../../hooks/useTranslation.js';
import { didsList } from '../../messaging.js';
import { Header } from '../../partials/index.js';
import { styled } from '../../styled.js';
import AccountsTree from './AccountsTree.js';
import AddAccount from './AddAccount.js';
import AddAccountImage from './AddAccountImage.js';

interface Props extends ThemeProps {
  className?: string;
}

function Accounts ({ className }: Props): React.ReactElement {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState<'accounts' | 'dids'>('accounts');
  const [filter, setFilter] = useState('');
  const [filteredAccount, setFilteredAccount] = useState<AccountWithChildren[]>([]);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const { hierarchy } = useContext(AccountContext);
  const onAction = useContext(ActionContext);
  const networkMap = useMemo(() => getNetworkMap(), []);
  const isAccountsTab = activeTab === 'accounts';
  const _onGenerateDids = useCallback(() => onAction('/did/create'), [onAction]);

  useEffect(() => {
    setFilteredAccount(
      filter
        ? hierarchy.filter((account) =>
          account.name?.toLowerCase().includes(filter) ||
          (account.genesisHash && networkMap.get(account.genesisHash)?.toLowerCase().includes(filter)) ||
          account.address.toLowerCase().includes(filter)
        )
        : hierarchy
    );
  }, [filter, hierarchy, networkMap]);

  useEffect(() => {
    if (activeTab !== 'dids') {
      return;
    }

    didsList()
      .then(setDids)
      .catch(console.error);
  }, [activeTab]);

  const _onFilter = useCallback((filter: string) => {
    setFilter(filter.toLowerCase());
  }, []);

  return (
    <>
      {(hierarchy.length === 0)
        ? <AddAccount />
        : (
          <>
            <Header
              onFilter={isAccountsTab ? _onFilter : undefined}
              showAdd={isAccountsTab}
              showConnectedAccounts={isAccountsTab}
              showSearch={isAccountsTab}
              showSettings
              text={isAccountsTab ? t<string>('Accounts') : 'DIDs'}
            />
            <div className={className}>
              <div className='tabs'>
                <button
                  className={`tab ${isAccountsTab ? 'isActive' : ''}`}
                  onClick={() => setActiveTab('accounts')}
                  type='button'
                >
                  {t<string>('Accounts')}
                </button>
                <button
                  className={`tab ${!isAccountsTab ? 'isActive' : ''}`}
                  onClick={() => setActiveTab('dids')}
                  type='button'
                >
                  DIDs
                </button>
              </div>
              <div className='content'>
                {isAccountsTab
                  ? (
                    <>
                      {filteredAccount.map((json, index): React.ReactNode => (
                        <AccountsTree
                          {...json}
                          key={`${index}:${json.address}`}
                        />
                      ))}
                    </>
                  )
                  : dids.length === 0
                    ? (
                      <div className='didsEmpty'>
                        <div className='image'>
                          <AddAccountImage onClick={_onGenerateDids} />
                        </div>
                        <div className='no-dids'>
                          <h3>Generate DIDs</h3>
                          <p>You currently don&apos;t have any DIDs. Generate your first DID to get started.</p>
                        </div>
                      </div>
                    )
                    : (
                      <div className='didsList'>
                        <Button
                          className='generateDid'
                          onClick={_onGenerateDids}
                        >
                          {t<string>('Generate DID')}
                        </Button>
                        {dids.map(({ did, name }) => (
                          <div
                            className='didItem'
                            key={did}
                          >
                            <div className='didName'>{name || t<string>('DID')}</div>
                            <div className='didValue'>{did}</div>
                          </div>
                        ))}
                      </div>
                    )
                }
              </div>
            </div>
          </>
        )
      }
    </>
  );
}

export default styled(Accounts)(({ theme }: Props) => `
  display: flex;
  flex-direction: column;
  .tabs {
    display: flex;
    gap: 8px;
    margin: -12px 16px 12px;
  }

  .tab {
    background: ${theme.inputBackground};
    border: 1px solid ${theme.inputBorderColor};
    border-radius: 999px;
    color: ${theme.labelColor};
    cursor: pointer;
    font-family: ${theme.fontFamily};
    font-size: 12px;
    padding: 6px 12px;
  }

  .tab.isActive {
    background: ${theme.buttonBackground};
    border-color: ${theme.buttonBackground};
    color: ${theme.buttonTextColor};
  }

  .content {
    flex: 1;
    overflow-y: scroll;
    scrollbar-width: none;
  }

  .content::-webkit-scrollbar {
    display: none;
  }

  .didsEmpty {
    color: ${theme.textColor};
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    text-align: center;
  }

  .didsList {
    padding: 0 16px 16px;
  }

  .didsList .generateDid {
    margin-bottom: 12px;
  }

  .didItem {
    background: ${theme.readonlyInputBackground};
    border: 1px solid ${theme.inputBorderColor};
    border-radius: ${theme.borderRadius};
    padding: 10px 12px;
    margin-bottom: 10px;
  }

  .didName {
    color: ${theme.labelColor};
    font-size: 12px;
    margin-bottom: 6px;
  }

  .didValue {
    color: ${theme.textColor};
    font-size: 13px;
    word-break: break-all;
  }

  .didsEmpty h3 {
    color: ${theme.textColor};
    margin-top: 0;
    font-weight: normal;
    font-size: 24px;
    line-height: 33px;
  }

  .didsEmpty .no-dids p {
    font-size: 16px;
    line-height: 26px;
    margin: 0 30px;
    color: ${theme.subTextColor};
  }

  height: calc(100vh - 2px);
  margin-top: -25px;
  padding-top: 25px;
`);
