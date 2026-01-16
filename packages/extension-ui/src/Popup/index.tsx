// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { AccountJson, AccountsContext, AuthorizeRequest, DidSigningRequest, MetadataRequest, SigningRequest } from '@polkadot/extension-base/background/types';
import type { SettingsStruct } from '@polkadot/ui-settings/types';

import React, { useCallback, useEffect, useState } from 'react';
import { Route, Switch, useHistory } from 'react-router';

import { PHISHING_PAGE_REDIRECT } from '@polkadot/extension-base/defaults';
import { canDerive } from '@polkadot/extension-base/utils';
import uiSettings from '@polkadot/ui-settings';

import { AccountContext, ActionContext, AuthorizeReqContext, DidSigningReqContext, MediaContext, MetadataReqContext, SettingsContext, SigningReqContext } from '../components/contexts.js';
import { ErrorBoundary, Loading } from '../components/index.js';
import ToastProvider from '../components/Toast/ToastProvider.js';
import { subscribeAccounts, subscribeAuthorizeRequests, subscribeDidSigningRequests, subscribeMetadataRequests, subscribeSigningRequests } from '../messaging.js';
import { buildHierarchy } from '../util/buildHierarchy.js';
import Accounts from './Accounts/index.js';
import AccountManagement from './AuthManagement/AccountManagement.js';
import AuthList from './AuthManagement/index.js';
import DidAuthList from './AuthManagement/DidAuthList.js';
import DidManagement from './AuthManagement/DidManagement.js';
import Authorize from './Authorize/index.js';
import CreateAccount from './CreateAccount/index.js';
import Derive from './Derive/index.js';
import CreateDid from './Dids/CreateDid.js';
import DeactivateDid from './Dids/DeactivateDid.js';
import ExportDid from './Dids/ExportDid.js';
import ImportSeed from './ImportSeed/index.js';
import Metadata from './Metadata/index.js';
import DidSigning from './DidSigning/index.js';
import Signing from './Signing/index.js';
import Export from './Export.js';
import ExportAll from './ExportAll.js';
import Forget from './Forget.js';
import ImportLedger from './ImportLedger.js';
import ImportQr from './ImportQr.js';
import PhishingDetected from './PhishingDetected.js';
import RestoreJson from './RestoreJson.js';
import Welcome from './Welcome.js';

const startSettings = uiSettings.get();

// Request permission for video, based on access we can hide/show import
async function requestMediaAccess (cameraOn: boolean): Promise<boolean> {
  if (!cameraOn) {
    return false;
  }

  try {
    await navigator.mediaDevices.getUserMedia({ video: true });

    return true;
  } catch (error) {
    console.error('Permission for video declined', (error as Error).message);
  }

  return false;
}

function initAccountContext ({ accounts, selectedAccounts, setSelectedAccounts }: Omit<AccountsContext, 'hierarchy' | 'master'>): AccountsContext {
  const hierarchy = buildHierarchy(accounts);
  const master = hierarchy.find(({ isExternal, type }) => !isExternal && canDerive(type));

  return {
    accounts,
    hierarchy,
    master,
    selectedAccounts,
    setSelectedAccounts
  };
}

export default function Popup (): React.ReactElement {
  const [accounts, setAccounts] = useState<null | AccountJson[]>(null);
  const [accountCtx, setAccountCtx] = useState<AccountsContext>({ accounts: [], hierarchy: [] });
  const [selectedAccounts, setSelectedAccounts] = useState<AccountJson['address'][]>([]);
  const [authRequests, setAuthRequests] = useState<null | AuthorizeRequest[]>(null);
  const [cameraOn, setCameraOn] = useState(startSettings.camera === 'on');
  const [mediaAllowed, setMediaAllowed] = useState(false);
  const [metaRequests, setMetaRequests] = useState<null | MetadataRequest[]>(null);
  const [signRequests, setSignRequests] = useState<null | SigningRequest[]>(null);
  const [didSignRequests, setDidSignRequests] = useState<null | DidSigningRequest[]>(null);
  const [isWelcomeDone, setWelcomeDone] = useState(false);
  const [settingsCtx, setSettingsCtx] = useState<SettingsStruct>(startSettings);
  const history = useHistory();

  const _onAction = useCallback(
    (to?: string): void => {
      setWelcomeDone(window.localStorage.getItem('welcome_read') === 'ok');

      if (!to) {
        return;
      }

      to === '../index.js'
        // if we can't go gack from there, go to the home
        ? history.length === 1
          ? history.push('/')
          : history.goBack()
        : window.location.hash = to;
    },
    [history]
  );

  useEffect((): void => {
    Promise.all([
      subscribeAccounts(setAccounts),
      subscribeAuthorizeRequests(setAuthRequests),
      subscribeMetadataRequests(setMetaRequests),
      subscribeSigningRequests(setSignRequests),
      subscribeDidSigningRequests(setDidSignRequests)
    ]).catch(console.error);

    uiSettings.on('change', (settings): void => {
      setSettingsCtx(settings);
      setCameraOn(settings.camera === 'on');
    });

    _onAction();
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect((): void => {
    setAccountCtx(initAccountContext({ accounts: accounts || [], selectedAccounts, setSelectedAccounts }));
  }, [accounts, selectedAccounts]);

  useEffect((): void => {
    requestMediaAccess(cameraOn)
      .then(setMediaAllowed)
      .catch(console.error);
  }, [cameraOn]);

  function wrapWithErrorBoundary (component: React.ReactElement, trigger?: string): React.ReactElement {
    return <ErrorBoundary trigger={trigger}>{component}</ErrorBoundary>;
  }

  const Root = isWelcomeDone
    ? authRequests && authRequests.length
      ? wrapWithErrorBoundary(<Authorize />, 'authorize')
      : metaRequests && metaRequests.length
        ? wrapWithErrorBoundary(<Metadata />, 'metadata')
        : didSignRequests && didSignRequests.length
          ? wrapWithErrorBoundary(<DidSigning />, 'did-signing')
          : signRequests && signRequests.length
            ? wrapWithErrorBoundary(<Signing />, 'signing')
            : wrapWithErrorBoundary(<Accounts />, 'accounts')
    : wrapWithErrorBoundary(<Welcome />, 'welcome');

  return (
    <Loading>{accounts && authRequests && metaRequests && signRequests && didSignRequests && (
      <ActionContext.Provider value={_onAction}>
        <SettingsContext.Provider value={settingsCtx}>
          <AccountContext.Provider value={accountCtx}>
            <AuthorizeReqContext.Provider value={authRequests}>
              <MediaContext.Provider value={cameraOn && mediaAllowed}>
                <MetadataReqContext.Provider value={metaRequests}>
                  <SigningReqContext.Provider value={signRequests}>
                    <DidSigningReqContext.Provider value={didSignRequests}>
                      <ToastProvider>
                        <Switch>
                          <Route path='/auth-list'>{wrapWithErrorBoundary(<AuthList />, 'auth-list')}</Route>
                          <Route path='/did/auth-list'>{wrapWithErrorBoundary(<DidAuthList />, 'did-auth-list')}</Route>
                          <Route path='/did/manage/:url'>{wrapWithErrorBoundary(<DidManagement />, 'did-manage-url')}</Route>
                          <Route path='/account/create'>{wrapWithErrorBoundary(<CreateAccount />, 'account-creation')}</Route>
                          <Route path='/account/forget/:address'>{wrapWithErrorBoundary(<Forget />, 'forget-address')}</Route>
                          <Route path='/account/export/:address'>{wrapWithErrorBoundary(<Export />, 'export-address')}</Route>
                          <Route path='/account/export-all'>{wrapWithErrorBoundary(<ExportAll />, 'export-all-address')}</Route>
                          <Route path='/account/import-ledger'>{wrapWithErrorBoundary(<ImportLedger />, 'import-ledger')}</Route>
                          <Route path='/account/import-qr'>{wrapWithErrorBoundary(<ImportQr />, 'import-qr')}</Route>
                          <Route path='/account/import-seed'>{wrapWithErrorBoundary(<ImportSeed />, 'import-seed')}</Route>
                          <Route path='/account/restore-json'>{wrapWithErrorBoundary(<RestoreJson />, 'restore-json')}</Route>
                          <Route path='/account/derive/:address/locked'>{wrapWithErrorBoundary(<Derive isLocked />, 'derived-address-locked')}</Route>
                          <Route path='/account/derive/:address'>{wrapWithErrorBoundary(<Derive />, 'derive-address')}</Route>
                          <Route path='/did/create'>{wrapWithErrorBoundary(<CreateDid />, 'did-create')}</Route>
                          <Route path='/did/deactivate/:did'>{wrapWithErrorBoundary(<DeactivateDid />, 'did-deactivate')}</Route>
                          <Route path='/did/export/:did'>{wrapWithErrorBoundary(<ExportDid />, 'did-export')}</Route>
                          <Route path='/url/manage/:url'>{wrapWithErrorBoundary(<AccountManagement />, 'manage-url')}</Route>
                          <Route path={`${PHISHING_PAGE_REDIRECT}/:website`}>{wrapWithErrorBoundary(<PhishingDetected />, 'phishing-page-redirect')}</Route>
                          <Route
                            exact
                            path='/'
                          >
                            {Root}
                          </Route>
                        </Switch>
                      </ToastProvider>
                    </DidSigningReqContext.Provider>
                  </SigningReqContext.Provider>
                </MetadataReqContext.Provider>
              </MediaContext.Provider>
            </AuthorizeReqContext.Provider>
          </AccountContext.Provider>
        </SettingsContext.Provider>
      </ActionContext.Provider>
    )}</Loading>
  );
}
