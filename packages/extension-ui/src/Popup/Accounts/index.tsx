// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { AccountWithChildren, DidRecord } from '@polkadot/extension-base/background/types';
import type { ThemeProps } from '../../types.js';

import { faCopy } from '@fortawesome/free-regular-svg-icons';
import { faPlusCircle } from '@fortawesome/free-solid-svg-icons';
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome';
import CopyToClipboard from 'react-copy-to-clipboard';
import React, { useCallback, useContext, useEffect, useMemo, useRef, useState } from 'react';

import getNetworkMap from '@polkadot/extension-ui/util/getNetworkMap';

import details from '../../assets/details.svg';
import { AccountContext, ActionContext, Link, MenuDivider, MenuItem } from '../../components/index.js';
import Menu from '../../components/Menu.js';
import Svg from '../../components/Svg.js';
import useOutsideClick from '../../hooks/useOutsideClick.js';
import useToast from '../../hooks/useToast.js';
import useTranslation from '../../hooks/useTranslation.js';
import { didsList, removeDid } from '../../messaging.js';
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
  const [activeTab, setActiveTab] = useState<'accounts' | 'dids'>(() => {
    const stored = window.localStorage.getItem('accounts_active_tab');

    return stored === 'dids' ? 'dids' : 'accounts';
  });
  const [filter, setFilter] = useState('');
  const [filteredAccount, setFilteredAccount] = useState<AccountWithChildren[]>([]);
  const [dids, setDids] = useState<DidRecord[]>([]);
  const [didMenuOpen, setDidMenuOpen] = useState<string | null>(null);
  const { hierarchy } = useContext(AccountContext);
  const onAction = useContext(ActionContext);
  const { show } = useToast();
  const networkMap = useMemo(() => getNetworkMap(), []);
  const isAccountsTab = activeTab === 'accounts';
  const _onGenerateDids = useCallback(() => onAction('/did/create'), [onAction]);
  const didMenuRef = useRef<HTMLDivElement>(null);
  const didMenuToggleRef = useRef<HTMLDivElement>(null);

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

  useOutsideClick([didMenuRef, didMenuToggleRef], () => {
    if (didMenuOpen) {
      setDidMenuOpen(null);
    }
  });

  useEffect(() => {
    if (activeTab !== 'dids') {
      return;
    }

    didsList()
      .then(setDids)
      .catch(console.error);
  }, [activeTab]);

  useEffect(() => {
    window.localStorage.setItem('accounts_active_tab', activeTab);
  }, [activeTab]);

  const _onFilter = useCallback((filter: string) => {
    setFilter(filter.toLowerCase());
  }, []);

  const _toggleDidMenu = useCallback((did: string) => {
    setDidMenuOpen((current) => (current === did ? null : did));
  }, []);

  const _onCopy = useCallback(() => show(t<string>('Copied')), [show, t]);

  const _onRemoveDid = useCallback((did: string) => {
    removeDid(did)
      .then(() => didsList().then(setDids))
      .catch(console.error)
      .finally(() => setDidMenuOpen(null));
  }, [setDids]);

  return (
    <>
      {(hierarchy.length === 0)
        ? <AddAccount />
        : (
          <>
            <Header
              onFilter={isAccountsTab ? _onFilter : undefined}
              showAdd
              showConnectedAccounts
              showSearch={isAccountsTab}
              connectedPathSingle={isAccountsTab ? '/url/manage' : '/did/manage'}
              connectedPathMulti={isAccountsTab ? '/auth-list' : '/did/auth-list'}
              addMenuItems={isAccountsTab ? undefined : (
                <MenuItem className='menuItem'>
                  <Link to='/did/create'>
                    <FontAwesomeIcon icon={faPlusCircle} />
                    <span>{t<string>('Create DID')}</span>
                  </Link>
                </MenuItem>
              )}
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
              <div className={`content ${isAccountsTab ? 'contentScroll' : 'contentNoScroll'}`}>
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
                      <div className='didsEmpty didsScroll'>
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
                      <>
                        <div className='didsList didsScroll'>
                          {dids.map(({ did, name, deactivated }) => (
                            <div
                              className='didItem'
                              key={did}
                            >
                              <div className='didIcon' aria-hidden='true'>
                                <span>ID</span>
                              </div>
                              <div className='didBody'>
                                <div className='didHeader'>
                                  <div className='didName'>{name || t<string>('DID')}</div>
                                  <div className={`didStatus ${deactivated === true ? 'inactive' : deactivated === false ? 'active' : 'unknown'}`}>
                                    {deactivated === true
                                      ? t<string>('Deactivated')
                                      : deactivated === false
                                        ? t<string>('Active')
                                        : t<string>('Unknown')}
                                  </div>
                                </div>
                                <div className='didValue'>
                                  <span className='didText'>{did}</span>
                                  <CopyToClipboard text={did}>
                                    <FontAwesomeIcon
                                      className='copyIcon'
                                      icon={faCopy}
                                      onClick={_onCopy}
                                      size='sm'
                                      title={t<string>('copy DID')}
                                    />
                                  </CopyToClipboard>
                                </div>
                              </div>
                              <div className='didActions'>
                                <div
                                  className='didActionsToggle'
                                  onClick={() => _toggleDidMenu(did)}
                                  ref={didMenuOpen === did ? didMenuToggleRef : undefined}
                                >
                                  <Svg
                                    className={`detailsIcon ${didMenuOpen === did ? 'active' : ''}`}
                                    src={details}
                                  />
                                </div>
                                {didMenuOpen === did && (
                                  <Menu
                                    className='didMenu'
                                    reference={didMenuRef}
                                  >
                                    <Link
                                      className='menuItem'
                                      isDisabled={deactivated === true}
                                      to={deactivated === true ? undefined : `/did/deactivate/${encodeURIComponent(did)}`}
                                    >
                                      {t<string>('Deactivate DID')}
                                    </Link>
                                    <MenuDivider />
                                    <Link
                                      className='menuItem'
                                      to={`/did/export/${encodeURIComponent(did)}`}
                                    >
                                      {t<string>('Download DID')}
                                    </Link>
                                    <MenuDivider />
                                    <Link
                                      className='menuItem'
                                      isDanger
                                      onClick={() => _onRemoveDid(did)}
                                    >
                                      {t<string>('Remove from wallet')}
                                    </Link>
                                  </Menu>
                                )}
                              </div>
                            </div>
                          ))}
                        </div>
                      </>
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
  overflow: hidden;
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
    display: flex;
    flex-direction: column;
    scrollbar-width: none;
  }

  .contentScroll {
    overflow-y: scroll;
  }

  .contentNoScroll {
    overflow: hidden;
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
    flex: 1;
    text-align: center;
  }

  .didsList {
    padding: 0 16px 16px;
  }

  .didsList .generateDid {
    margin-bottom: 12px;
  }

  .didsScroll {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: none;
    padding-bottom: 24px;
  }

  .didsScroll::-webkit-scrollbar {
    display: none;
  }

  .didItem {
    background: ${theme.readonlyInputBackground};
    border: 1px solid ${theme.inputBorderColor};
    border-radius: ${theme.borderRadius};
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    margin-bottom: 10px;
    position: relative;
  }

  .didName {
    color: ${theme.textColor};
    font-size: 14px;
    margin-bottom: 4px;
    font-weight: 600;
  }

  .didHeader {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .didStatus {
    border-radius: 999px;
    border: 1px solid ${theme.inputBorderColor};
    color: ${theme.labelColor};
    font-size: 10px;
    line-height: 14px;
    padding: 1px 8px;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .didStatus.active {
    border-color: #2e7d6a;
    color: ${theme.textColor};
    background: rgba(46, 125, 106, 0.18);
  }

  .didStatus.inactive {
    border-color: #a14b4b;
    color: ${theme.textColor};
    background: rgba(161, 75, 75, 0.18);
  }

  .didStatus.unknown {
    border-color: ${theme.inputBorderColor};
    color: ${theme.labelColor};
    background: rgba(255, 255, 255, 0.04);
  }

  .didValue {
    color: ${theme.labelColor};
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .didText {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .copyIcon {
    color: ${theme.iconNeutralColor};
    cursor: pointer;
    flex: 0 0 auto;
    align-self: center;
  }

  .didIcon {
    width: 32px;
    height: 32px;
    border-radius: 999px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: ${theme.buttonBackground};
    color: ${theme.buttonTextColor};
    font-size: 16px;
    flex: 0 0 32px;
  }

  .didItem:nth-child(6n + 1) .didIcon {
    background: #2e7d6a;
    color: ${theme.textColor};
  }

  .didItem:nth-child(6n + 2) .didIcon {
    background: #b8862a;
    color: ${theme.bodyColor};
  }

  .didItem:nth-child(6n + 3) .didIcon {
    background: #a14b4b;
    color: ${theme.textColor};
  }

  .didItem:nth-child(6n + 4) .didIcon {
    background: #3a5c8a;
    color: ${theme.textColor};
  }

  .didItem:nth-child(6n + 5) .didIcon {
    background: #4c5a6e;
    color: ${theme.textColor};
  }

  .didItem:nth-child(6n) .didIcon {
    background: ${theme.buttonBackground};
    color: ${theme.buttonTextColor};
  }

  .didBody {
    min-width: 0;
    flex: 1;
  }

  .didActions {
    align-items: center;
    display: inline-flex;
    height: 100%;
    justify-content: center;
    width: 32px;
  }

  .didActionsToggle {
    align-items: center;
    cursor: pointer;
    display: inline-flex;
    height: 100%;
    justify-content: center;
    width: 32px;
  }

  .detailsIcon {
    background: ${theme.accountDotsIconColor};
    width: 3px;
    height: 19px;

    &.active {
      background: ${theme.primaryColor};
    }
  }

  .didMenu {
    margin-top: 6px;
    right: 8px;
    top: 100%;
  }

  .menuItem {
    border-radius: 8px;
    display: block;
    font-size: 15px;
    line-height: 20px;
    margin: 0;
    min-width: 12rem;
    padding: 4px 16px;
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
