// Copyright 2019-2023 @polkadot/extension-base authors & contributors
// SPDX-License-Identifier: Apache-2.0

/* global chrome */

import type { MetadataDef } from '@polkadot/extension-inject/types';
import type { KeyringPair, KeyringPair$Json, KeyringPair$Meta } from '@polkadot/keyring/types';
import type { Registry, SignerPayloadJSON, SignerPayloadRaw } from '@polkadot/types/types';
import type { SubjectInfo } from '@polkadot/ui-keyring/observable/types';
import type { KeypairType } from '@polkadot/util-crypto/types';
import type { AccountJson, AllowedPath, AuthorizeRequest, DidRecord, DidSigningRequest, MessageTypes, MetadataRequest, RequestAccountBatchExport, RequestAccountChangePassword, RequestAccountCreateExternal, RequestAccountCreateHardware, RequestAccountCreateSuri, RequestAccountEdit, RequestAccountExport, RequestAccountForget, RequestAccountShow, RequestAccountTie, RequestAccountValidate, RequestActiveTabsUrlUpdate, RequestAuthorizeApprove, RequestBatchRestore, RequestDeriveCreate, RequestDeriveValidate, RequestDidCreate, RequestDidDeactivate, RequestDidExport, RequestDidRemove, RequestDidSignApprove, RequestDidSignCancel, RequestJsonRestore, RequestMetadataApprove, RequestMetadataReject, RequestSeedCreate, RequestSeedValidate, RequestSigningApprovePassword, RequestSigningApproveSignature, RequestSigningCancel, RequestSigningIsLocked, RequestTypes, RequestUpdateAuthorizedAccounts, ResponseAccountExport, ResponseAccountsExport, ResponseAuthorizeList, ResponseDeriveValidate, ResponseDidExport, ResponseJsonGetAccountInfo, ResponseSeedCreate, ResponseSeedValidate, ResponseSigningIsLocked, ResponseType, SigningRequest } from '../types.js';
import type { AuthorizedAccountsDiff } from './State.js';
import type State from './State.js';

import { ApiPromise, WsProvider } from '@polkadot/api';
import { ALLOWED_PATH, PASSWORD_EXPIRY_MS } from '@polkadot/extension-base/defaults';
import { metadataExpand } from '@polkadot/extension-chains';
import { Keyring } from '@polkadot/keyring';
import { TypeRegistry } from '@polkadot/types';
import keyring from '@polkadot/ui-keyring';
import { accounts as accountsObservable } from '@polkadot/ui-keyring/observable/accounts';
import { assert, isHex, stringToU8a, u8aConcat, u8aToHex } from '@polkadot/util';
import { base58Decode, base58Encode, blake2AsU8a, cryptoWaitReady, keyExtractSuri, mldsa44PairFromSeed, mnemonicGenerate, mnemonicValidate, randomAsU8a } from '@polkadot/util-crypto';

import { withErrorLog } from './helpers.js';
import { createSubscription, unsubscribe } from './subscriptions.js';
import { DidsStore } from '../../stores/index.js';

type CachedUnlocks = Record<string, number>;

const SEED_DEFAULT_LENGTH = 12;
const SEED_LENGTHS = [12, 15, 18, 21, 24];
const ETH_DERIVE_DEFAULT = "/m/44'/60'/0'/0/0";
const QSB_POSEIDON_ENDPOINT = 'wss://qsb.qbck.io:9945';

function getSuri (seed: string, type?: KeypairType): string {
  return type === 'ethereum'
    ? `${seed}${ETH_DERIVE_DEFAULT}`
    : seed;
}

function isJsonPayload (value: SignerPayloadJSON | SignerPayloadRaw): value is SignerPayloadJSON {
  return (value as SignerPayloadJSON).genesisHash !== undefined;
}

export default class Extension {
  readonly #cachedUnlocks: CachedUnlocks;

  readonly #state: State;
  readonly #didsStore: DidsStore;

  constructor (state: State) {
    this.#cachedUnlocks = {};
    this.#state = state;
    this.#didsStore = new DidsStore();
  }

  private transformAccounts (accounts: SubjectInfo): AccountJson[] {
    return Object.values(accounts).map(({ json: { address, meta }, type }): AccountJson => ({
      address,
      isDefaultAuthSelected: this.#state.defaultAuthAccountSelection.includes(address),
      ...meta,
      type
    }));
  }

  private accountsCreateExternal ({ address, genesisHash, name }: RequestAccountCreateExternal): boolean {
    keyring.addExternal(address, { genesisHash, name });

    return true;
  }

  private accountsCreateHardware ({ accountIndex, address, addressOffset, genesisHash, hardwareType, name }: RequestAccountCreateHardware): boolean {
    keyring.addHardware(address, hardwareType, { accountIndex, addressOffset, genesisHash, name });

    return true;
  }

  private accountsCreateSuri ({ genesisHash, name, password, suri, type }: RequestAccountCreateSuri): boolean {
    keyring.addUri(getSuri(suri, type), password, { genesisHash, name }, type);

    return true;
  }

  private accountsChangePassword ({ address, newPass, oldPass }: RequestAccountChangePassword): boolean {
    const pair = keyring.getPair(address);

    assert(pair, 'Unable to find pair');

    try {
      if (!pair.isLocked) {
        pair.lock();
      }

      pair.decodePkcs8(oldPass);
    } catch {
      throw new Error('oldPass is invalid');
    }

    keyring.encryptAccount(pair, newPass);

    return true;
  }

  private accountsEdit ({ address, name }: RequestAccountEdit): boolean {
    const pair = keyring.getPair(address);

    assert(pair, 'Unable to find pair');

    keyring.saveAccountMeta(pair, { ...pair.meta, name });

    return true;
  }

  private accountsExport ({ address, password }: RequestAccountExport): ResponseAccountExport {
    return { exportedJson: keyring.backupAccount(keyring.getPair(address), password) };
  }

  private async accountsBatchExport ({ addresses, password }: RequestAccountBatchExport): Promise<ResponseAccountsExport> {
    return {
      exportedJson: await keyring.backupAccounts(addresses, password)
    };
  }

  private accountsForget ({ address }: RequestAccountForget): boolean {
    const authorizedAccountsDiff: AuthorizedAccountsDiff = [];

    // cycle through authUrls and prepare the array of diff
    Object.entries(this.#state.authUrls).forEach(([url, urlInfo]) => {
      if (!urlInfo.authorizedAccounts.includes(address)) {
        return;
      }

      authorizedAccountsDiff.push([url, urlInfo.authorizedAccounts.filter((previousAddress) => previousAddress !== address)]);
    });

    this.#state.updateAuthorizedAccounts(authorizedAccountsDiff);

    // cycle through default account selection for auth and remove any occurence of the account
    const newDefaultAuthAccounts = this.#state.defaultAuthAccountSelection.filter((defaultSelectionAddress) => defaultSelectionAddress !== address);

    this.#state.updateDefaultAuthAccounts(newDefaultAuthAccounts);

    keyring.forgetAccount(address);

    return true;
  }

  private refreshAccountPasswordCache (pair: KeyringPair): number {
    const { address } = pair;

    const savedExpiry = this.#cachedUnlocks[address] || 0;
    const remainingTime = savedExpiry - Date.now();

    if (remainingTime < 0) {
      this.#cachedUnlocks[address] = 0;
      pair.lock();

      return 0;
    }

    return remainingTime;
  }

  private accountsShow ({ address, isShowing }: RequestAccountShow): boolean {
    const pair = keyring.getPair(address);

    assert(pair, 'Unable to find pair');

    keyring.saveAccountMeta(pair, { ...pair.meta, isHidden: !isShowing });

    return true;
  }

  private accountsTie ({ address, genesisHash }: RequestAccountTie): boolean {
    const pair = keyring.getPair(address);

    assert(pair, 'Unable to find pair');

    keyring.saveAccountMeta(pair, { ...pair.meta, genesisHash });

    return true;
  }

  private accountsValidate ({ address, password }: RequestAccountValidate): boolean {
    try {
      keyring.backupAccount(keyring.getPair(address), password);

      return true;
    } catch {
      return false;
    }
  }

  private accountsSubscribe (id: string, port: chrome.runtime.Port): boolean {
    const cb = createSubscription<'pri(accounts.subscribe)'>(id, port);
    const subscription = accountsObservable.subject.subscribe((accounts: SubjectInfo): void =>
      cb(this.transformAccounts(accounts))
    );

    port.onDisconnect.addListener((): void => {
      unsubscribe(id);
      subscription.unsubscribe();
    });

    return true;
  }

  private authorizeApprove ({ authorizedAccounts, authorizedDids, id }: RequestAuthorizeApprove): boolean {
    const queued = this.#state.getAuthRequest(id);

    assert(queued, 'Unable to find request');

    const { resolve } = queued;

    resolve({ authorizedAccounts, authorizedDids, result: true });

    return true;
  }

  private authorizeUpdate ({ authorizedAccounts, authorizedDids, url }: RequestUpdateAuthorizedAccounts): void {
    return this.#state.updateAuthorizedAccounts([[url, authorizedAccounts, authorizedDids]]);
  }

  private getAuthList (): ResponseAuthorizeList {
    return { list: this.#state.authUrls };
  }

  // FIXME This looks very much like what we have in accounts
  private authorizeSubscribe (id: string, port: chrome.runtime.Port): boolean {
    const cb = createSubscription<'pri(authorize.requests)'>(id, port);
    const subscription = this.#state.authSubject.subscribe((requests: AuthorizeRequest[]): void =>
      cb(requests)
    );

    port.onDisconnect.addListener((): void => {
      unsubscribe(id);
      subscription.unsubscribe();
    });

    return true;
  }

  private metadataApprove ({ id }: RequestMetadataApprove): boolean {
    const queued = this.#state.getMetaRequest(id);

    assert(queued, 'Unable to find request');

    const { request, resolve } = queued;

    this.#state.saveMetadata(request);

    resolve(true);

    return true;
  }

  private metadataGet (genesisHash: string | null): MetadataDef | null {
    return this.#state.knownMetadata.find((result) => result.genesisHash === genesisHash) || null;
  }

  private metadataList (): MetadataDef[] {
    return this.#state.knownMetadata;
  }

  private metadataReject ({ id }: RequestMetadataReject): boolean {
    const queued = this.#state.getMetaRequest(id);

    assert(queued, 'Unable to find request');

    const { reject } = queued;

    reject(new Error('Rejected'));

    return true;
  }

  private metadataSubscribe (id: string, port: chrome.runtime.Port): boolean {
    const cb = createSubscription<'pri(metadata.requests)'>(id, port);
    const subscription = this.#state.metaSubject.subscribe((requests: MetadataRequest[]): void =>
      cb(requests)
    );

    port.onDisconnect.addListener((): void => {
      unsubscribe(id);
      subscription.unsubscribe();
    });

    return true;
  }

  private jsonRestore ({ file, password }: RequestJsonRestore): void {
    try {
      keyring.restoreAccount(file, password);
    } catch (error) {
      throw new Error((error as Error).message);
    }
  }

  private batchRestore ({ file, password }: RequestBatchRestore): void {
    try {
      keyring.restoreAccounts(file, password);
    } catch (error) {
      throw new Error((error as Error).message);
    }
  }

  private jsonGetAccountInfo (json: KeyringPair$Json): ResponseJsonGetAccountInfo {
    try {
      const { address, meta: { genesisHash, name }, type } = keyring.createFromJson(json);

      return {
        address,
        genesisHash,
        name,
        type
      } as ResponseJsonGetAccountInfo;
    } catch (e) {
      console.error(e);
      throw new Error((e as Error).message);
    }
  }

  private seedCreate ({ length = SEED_DEFAULT_LENGTH, seed: _seed, type }: RequestSeedCreate): ResponseSeedCreate {
    const seed = _seed || mnemonicGenerate(length);

    return {
      address: keyring.createFromUri(getSuri(seed, type), {}, type).address,
      seed
    };
  }

  private seedValidate ({ suri, type }: RequestSeedValidate): ResponseSeedValidate {
    const { phrase } = keyExtractSuri(suri);

    if (isHex(phrase)) {
      assert(isHex(phrase, 256), 'Hex seed needs to be 256-bits');
    } else {
      // sadly isHex detects as string, so we need a cast here
      assert(SEED_LENGTHS.includes((phrase).split(' ').length), `Mnemonic needs to contain ${SEED_LENGTHS.join(', ')} words`);
      assert(mnemonicValidate(phrase), 'Not a valid mnemonic seed');
    }

    return {
      address: keyring.createFromUri(getSuri(suri, type), {}, type).address,
      suri
    };
  }

  private signingApprovePassword ({ id, password, savePass }: RequestSigningApprovePassword): boolean {
    const queued = this.#state.getSignRequest(id);

    assert(queued, 'Unable to find request');

    const { reject, resolve, request } = queued;
    const pair = keyring.getPair(queued.account.address);

    if (!pair) {
      reject(new Error('Unable to find pair'));

      return false;
    }

    this.refreshAccountPasswordCache(pair);

    // if the keyring pair is locked, the password is needed
    if (pair.isLocked && !password) {
      reject(new Error('Password needed to unlock the account'));
    }

    if (pair.isLocked) {
      pair.decodePkcs8(password);
    }

    // construct a new registry (avoiding pollution), between requests
    let registry: Registry;
    const { payload } = request;

    if (isJsonPayload(payload)) {
      // Get the metadata for the genesisHash
      const metadata = this.#state.knownMetadata.find(({ genesisHash }) => genesisHash === payload.genesisHash);

      if (metadata) {
        // we have metadata, expand it and extract the info/registry
        const expanded = metadataExpand(metadata, false);

        registry = expanded.registry;
        registry.setSignedExtensions(payload.signedExtensions, expanded.definition.userExtensions);
      } else {
        // we have no metadata, create a new registry
        registry = new TypeRegistry();
        registry.setSignedExtensions(payload.signedExtensions);
      }
    } else {
      // for non-payload, just create a registry to use
      registry = new TypeRegistry();
    }

    const result = request.sign(registry, pair);

    if (savePass) {
      // unlike queued.account.address the following
      // address is encoded with the default prefix
      // which what is used for password caching mapping
      this.#cachedUnlocks[pair.address] = Date.now() + PASSWORD_EXPIRY_MS;
    } else {
      pair.lock();
    }

    resolve({
      id,
      ...result
    });

    return true;
  }

  private signingApproveSignature ({ id, signature }: RequestSigningApproveSignature): boolean {
    const queued = this.#state.getSignRequest(id);

    assert(queued, 'Unable to find request');

    const { resolve } = queued;

    resolve({ id, signature });

    return true;
  }

  private signingCancel ({ id }: RequestSigningCancel): boolean {
    const queued = this.#state.getSignRequest(id);

    assert(queued, 'Unable to find request');

    const { reject } = queued;

    reject(new Error('Cancelled'));

    return true;
  }

  private signingIsLocked ({ id }: RequestSigningIsLocked): ResponseSigningIsLocked {
    const queued = this.#state.getSignRequest(id);

    assert(queued, 'Unable to find request');

    const address = queued.request.payload.address;
    const pair = keyring.getPair(address);

    assert(pair, 'Unable to find pair');

    const remainingTime = this.refreshAccountPasswordCache(pair);

    return {
      isLocked: pair.isLocked,
      remainingTime
    };
  }

  // FIXME This looks very much like what we have in authorization
  private signingSubscribe (id: string, port: chrome.runtime.Port): boolean {
    const cb = createSubscription<'pri(signing.requests)'>(id, port);
    const subscription = this.#state.signSubject.subscribe((requests: SigningRequest[]): void =>
      cb(requests)
    );

    port.onDisconnect.addListener((): void => {
      unsubscribe(id);
      subscription.unsubscribe();
    });

    return true;
  }

  private async didsSignApprove ({ id, password }: RequestDidSignApprove): Promise<boolean> {
    const queued = this.#state.getDidSignRequest(id);

    assert(queued, 'Unable to find DID signing request');

    const { reject, resolve } = queued;
    const didJson = await new Promise<KeyringPair$Json>((resolve, reject): void => {
      this.#didsStore.get(`did:${queued.did}`, (json): void => {
        if (!json) {
          reject(new Error('DID not found'));
        } else {
          resolve(json);
        }
      });
    });

    const didKeyring = new Keyring({ type: 'mldsa44' });
    const didPair = didKeyring.addFromJson(didJson);

    try {
      didPair.decodePkcs8(password);
    } catch {
      reject(new Error('Wrong DID password'));

      return false;
    }

    try {
      resolve({
        id,
        signature: '0x'
      });
    } finally {
      didPair.lock();
    }

    return true;
  }

  private didsSignCancel ({ id }: RequestDidSignCancel): boolean {
    const queued = this.#state.getDidSignRequest(id);

    assert(queued, 'Unable to find DID signing request');

    const { reject } = queued;

    reject(new Error('Cancelled'));

    return true;
  }

  private didsSignSubscribe (id: string, port: chrome.runtime.Port): boolean {
    const cb = createSubscription<'pri(dids.sign.requests)'>(id, port);
    const subscription = this.#state.didSignSubject.subscribe((requests: DidSigningRequest[]): void =>
      cb(requests)
    );

    port.onDisconnect.addListener((): void => {
      unsubscribe(id);
      subscription.unsubscribe();
    });

    return true;
  }

  private windowOpen (path: AllowedPath): boolean {
    const url = `${chrome.extension.getURL('index.html')}#${path}`;

    if (!ALLOWED_PATH.includes(path)) {
      console.error('Not allowed to open the url:', url);

      return false;
    }

    withErrorLog(() => chrome.tabs.create({ url }));

    return true;
  }

  private derive (parentAddress: string, suri: string, password: string, metadata: KeyringPair$Meta): KeyringPair {
    const parentPair = keyring.getPair(parentAddress);

    try {
      parentPair.decodePkcs8(password);
    } catch {
      throw new Error('invalid password');
    }

    try {
      return parentPair.derive(suri, metadata);
    } catch {
      throw new Error(`"${suri}" is not a valid derivation path`);
    }
  }

  private derivationValidate ({ parentAddress, parentPassword, suri }: RequestDeriveValidate): ResponseDeriveValidate {
    const childPair = this.derive(parentAddress, suri, parentPassword, {});

    return {
      address: childPair.address,
      suri
    };
  }

  private derivationCreate ({ genesisHash, name, parentAddress, parentPassword, password, suri }: RequestDeriveCreate): boolean {
    const childPair = this.derive(parentAddress, suri, parentPassword, {
      genesisHash,
      name,
      parentAddress,
      suri
    });

    keyring.addPair(childPair, password);

    return true;
  }

  private removeAuthorization (url: string): ResponseAuthorizeList {
    return { list: this.#state.removeAuthorization(url) };
  }

  private deleteAuthRequest (requestId: string): void {
    return this.#state.deleteAuthRequest(requestId);
  }

  private updateCurrentTabs ({ urls }: RequestActiveTabsUrlUpdate) {
    this.#state.udateCurrentTabsUrl(urls);
  }

  private getConnectedTabsUrl () {
    return this.#state.getConnectedTabsUrl();
  }

  private async didsCreate ({ accountAddress, name, accountPassword, didPassword }: RequestDidCreate): Promise<DidRecord> {
    await cryptoWaitReady();

    const signerPair = keyring.getPair(accountAddress);

    assert(signerPair, 'Unable to find signing account');
    assert(!signerPair.meta.isExternal && !signerPair.meta.isHardware, 'Account cannot sign transactions');

    if (signerPair.isLocked) {
      if (!accountPassword) {
        throw new Error('Password needed to unlock the account');
      }

      try {
        signerPair.decodePkcs8(accountPassword);
      } catch {
        throw new Error('Wrong password');
      }
    }

    const provider = new WsProvider(QSB_POSEIDON_ENDPOINT);
    const api = await ApiPromise.create({ provider });

    const didPair = mldsa44PairFromSeed(randomAsU8a(32));
    const genesisHash = api.genesisHash.toU8a();
    const didId = blake2AsU8a(
      u8aConcat(
        stringToU8a('QSB_DID'),
        genesisHash,
        didPair.publicKey
      ),
      256
    );
    const did = `did:qsb:${base58Encode(didId)}`;
    const publicKeyHex = u8aToHex(didPair.publicKey);
    const genesisHashHex = api.genesisHash.toHex();

    try {
      await new Promise<void>(async (resolve, reject) => {
        let unsub: (() => void) | undefined;

        try {
          unsub = await api.tx['did']
            ['createDid'](publicKeyHex, u8aToHex(new Uint8Array()))
            .signAndSend(signerPair, (result): void => {
              if (result.dispatchError) {
                if (unsub) {
                  unsub();
                }

                if (result.dispatchError.isModule) {
                  const decoded = api.registry.findMetaError(result.dispatchError.asModule);
                  const message = decoded.section && decoded.name
                    ? `${decoded.section}.${decoded.name}`
                    : decoded.name;

                  reject(new Error(message));
                } else {
                  reject(new Error(result.dispatchError.toString()));
                }

                return;
              }

              if (result.status.isInBlock || result.status.isFinalized) {
                if (unsub) {
                  unsub();
                }

                resolve();
              }
            });
        } catch (error) {
          if (unsub) {
            unsub();
          }

          reject(error as Error);
        }
      });
    } finally {
      signerPair.lock();
      await api.disconnect();
    }

    const didKeyring = new Keyring({ type: 'mldsa44' });
    const didKeypair = didKeyring.createFromPair(
      didPair,
      {
        accountAddress,
        did,
        genesisHash: genesisHashHex,
        name,
        publicKey: publicKeyHex
      },
      'mldsa44'
    );
    const json = didKeypair.toJson(didPassword);

    this.#didsStore.set(`did:${did}`, json);

    return {
      accountAddress,
      did,
      genesisHash: genesisHashHex,
      name,
      publicKey: publicKeyHex
    };
  }

  private async didsDeactivate ({ accountAddress, accountPassword, did, didPassword }: RequestDidDeactivate): Promise<boolean> {
    const didJson = await new Promise<KeyringPair$Json>((resolve, reject) => {
      this.#didsStore.get(`did:${did}`, (json) => {
        if (!json) {
          reject(new Error('DID not found'));
        } else {
          resolve(json);
        }
      });
    });
    const didKeyring = new Keyring({ type: 'mldsa44' });
    const didPair = didKeyring.addFromJson(didJson);

    try {
      didPair.decodePkcs8(didPassword);
    } catch {
      throw new Error('Wrong DID password');
    } finally {
      didPair.lock();
    }

    const signerPair = keyring.getPair(accountAddress);

    assert(signerPair, 'Unable to find signing account');
    assert(!signerPair.meta.isExternal && !signerPair.meta.isHardware, 'Account cannot sign transactions');

    if (signerPair.isLocked) {
      if (!accountPassword) {
        throw new Error('Password needed to unlock the account');
      }

      try {
        signerPair.decodePkcs8(accountPassword);
      } catch {
        throw new Error('Wrong password');
      }
    }

    const provider = new WsProvider(QSB_POSEIDON_ENDPOINT);
    const api = await ApiPromise.create({ provider });

    try {
      await new Promise<void>(async (resolve, reject) => {
        let unsub: (() => void) | undefined;

        try {
          unsub = await api.tx['did']
            ['deactivateDid'](did)
            .signAndSend(signerPair, (result): void => {
              if (result.dispatchError) {
                if (unsub) {
                  unsub();
                }

                if (result.dispatchError.isModule) {
                  const decoded = api.registry.findMetaError(result.dispatchError.asModule);
                  const message = decoded.section && decoded.name
                    ? `${decoded.section}.${decoded.name}`
                    : decoded.name;

                  reject(new Error(message));
                } else {
                  reject(new Error(result.dispatchError.toString()));
                }

                return;
              }

              if (result.status.isInBlock || result.status.isFinalized) {
                if (unsub) {
                  unsub();
                }

                resolve();
              }
            });
        } catch (error) {
          if (unsub) {
            unsub();
          }

          reject(error as Error);
        }
      });
    } finally {
      signerPair.lock();
      await api.disconnect();
    }

    return true;
  }

  private async didsExport ({ did, password }: RequestDidExport): Promise<ResponseDidExport> {
    const didJson = await new Promise<KeyringPair$Json>((resolve, reject) => {
      this.#didsStore.get(`did:${did}`, (json) => {
        if (!json) {
          reject(new Error('DID not found'));
        } else {
          resolve(json);
        }
      });
    });

    const didKeyring = new Keyring({ type: 'mldsa44' });
    const pair = didKeyring.addFromJson(didJson);

    try {
      pair.decodePkcs8(password);
    } catch {
      throw new Error('Wrong password');
    } finally {
      pair.lock();
    }

    return { exportedJson: didJson };
  }

  private async didsList (): Promise<DidRecord[]> {
    const records = await new Promise<DidRecord[]>((resolve) => {
      this.#didsStore.allMap((map) => {
        const records = Object.values(map)
          .map(({ meta }) => meta as unknown as Partial<DidRecord> | undefined)
          .filter((meta): meta is DidRecord =>
            Boolean(meta && meta.did && meta.accountAddress && meta.genesisHash && meta.publicKey)
          )
          .map((meta) => ({
            accountAddress: meta.accountAddress,
            did: meta.did,
            genesisHash: meta.genesisHash,
            name: meta.name,
            publicKey: meta.publicKey
          }));

        resolve(records);
      });
    });

    if (!records.length) {
      return records;
    }

    const didIds = records.map(({ did }) => {
      const normalized = did.startsWith('did:qsb:') ? did.slice(8) : did;

      try {
        return base58Decode(normalized);
      } catch {
        return null;
      }
    });

    const queries = didIds.map((id) => id && id.length === 32 ? id : null);
    const validIds = queries.filter((id): id is Uint8Array => !!id);

    if (!validIds.length) {
      return records;
    }

    try {
      const provider = new WsProvider(QSB_POSEIDON_ENDPOINT);
      const api = await ApiPromise.create({ provider });
      const results = await api.query['did']['didRecords'].multi(validIds);
      let index = 0;

      const withStatus = records.map((record, recordIndex) => {
        const didId = queries[recordIndex];

        if (!didId) {
          return record;
        }

        const result = results[index++] as unknown as { isNone?: boolean; unwrap?: () => { deactivated: { isTrue: boolean } } };

        if (!result || result.isNone) {
          return record;
        }

        return {
          ...record,
          deactivated: result.unwrap ? result.unwrap().deactivated.isTrue : undefined
        };
      });

      await api.disconnect();

      return withStatus;
    } catch (error) {
      console.error(error);
      return records;
    }
  }

  private async didsRemove ({ did }: RequestDidRemove): Promise<boolean> {
    return new Promise((resolve) => {
      this.#didsStore.remove(`did:${did}`, () => resolve(true));
    });
  }

  // Weird thought, the eslint override is not needed in Tabs
  // eslint-disable-next-line @typescript-eslint/require-await
  public async handle<TMessageType extends MessageTypes> (id: string, type: TMessageType, request: RequestTypes[TMessageType], port?: chrome.runtime.Port): Promise<ResponseType<TMessageType>> {
    switch (type) {
      case 'pri(authorize.approve)':
        return this.authorizeApprove(request as RequestAuthorizeApprove);

      case 'pri(authorize.list)':
        return this.getAuthList();

      case 'pri(authorize.remove)':
        return this.removeAuthorization(request as string);

      case 'pri(authorize.delete.request)':
        return this.deleteAuthRequest(request as string);

      case 'pri(authorize.requests)':
        return port && this.authorizeSubscribe(id, port);

      case 'pri(authorize.update)':
        return this.authorizeUpdate(request as RequestUpdateAuthorizedAccounts);

      case 'pri(accounts.create.external)':
        return this.accountsCreateExternal(request as RequestAccountCreateExternal);

      case 'pri(accounts.create.hardware)':
        return this.accountsCreateHardware(request as RequestAccountCreateHardware);

      case 'pri(accounts.create.suri)':
        return this.accountsCreateSuri(request as RequestAccountCreateSuri);

      case 'pri(accounts.changePassword)':
        return this.accountsChangePassword(request as RequestAccountChangePassword);

      case 'pri(accounts.edit)':
        return this.accountsEdit(request as RequestAccountEdit);

      case 'pri(accounts.export)':
        return this.accountsExport(request as RequestAccountExport);

      case 'pri(accounts.batchExport)':
        return this.accountsBatchExport(request as RequestAccountBatchExport);

      case 'pri(accounts.forget)':
        return this.accountsForget(request as RequestAccountForget);

      case 'pri(accounts.show)':
        return this.accountsShow(request as RequestAccountShow);

      case 'pri(accounts.subscribe)':
        return port && this.accountsSubscribe(id, port);

      case 'pri(accounts.tie)':
        return this.accountsTie(request as RequestAccountTie);

      case 'pri(accounts.validate)':
        return this.accountsValidate(request as RequestAccountValidate);

      case 'pri(metadata.approve)':
        return this.metadataApprove(request as RequestMetadataApprove);

      case 'pri(metadata.get)':
        return this.metadataGet(request as string);

      case 'pri(metadata.list)':
        return this.metadataList();

      case 'pri(metadata.reject)':
        return this.metadataReject(request as RequestMetadataReject);

      case 'pri(metadata.requests)':
        return port && this.metadataSubscribe(id, port);

      case 'pri(activeTabsUrl.update)':
        return this.updateCurrentTabs(request as RequestActiveTabsUrlUpdate);

      case 'pri(connectedTabsUrl.get)':
        return this.getConnectedTabsUrl();

      case 'pri(derivation.create)':
        return this.derivationCreate(request as RequestDeriveCreate);

      case 'pri(derivation.validate)':
        return this.derivationValidate(request as RequestDeriveValidate);

      case 'pri(dids.create)':
        return this.didsCreate(request as RequestDidCreate);

      case 'pri(dids.deactivate)':
        return this.didsDeactivate(request as RequestDidDeactivate);

      case 'pri(dids.export)':
        return this.didsExport(request as RequestDidExport);

      case 'pri(dids.sign.approve)':
        return this.didsSignApprove(request as RequestDidSignApprove);

      case 'pri(dids.sign.cancel)':
        return this.didsSignCancel(request as RequestDidSignCancel);

      case 'pri(dids.sign.requests)':
        return port && this.didsSignSubscribe(id, port);

      case 'pri(dids.remove)':
        return this.didsRemove(request as RequestDidRemove);

      case 'pri(dids.list)':
        return this.didsList();

      case 'pri(json.restore)':
        return this.jsonRestore(request as RequestJsonRestore);

      case 'pri(json.batchRestore)':
        return this.batchRestore(request as RequestBatchRestore);

      case 'pri(json.account.info)':
        return this.jsonGetAccountInfo(request as KeyringPair$Json);

      case 'pri(ping)':
        return Promise.resolve(true);

      case 'pri(seed.create)':
        return this.seedCreate(request as RequestSeedCreate);

      case 'pri(seed.validate)':
        return this.seedValidate(request as RequestSeedValidate);

      case 'pri(settings.notification)':
        return this.#state.setNotification(request as string);

      case 'pri(signing.approve.password)':
        return this.signingApprovePassword(request as RequestSigningApprovePassword);

      case 'pri(signing.approve.signature)':
        return this.signingApproveSignature(request as RequestSigningApproveSignature);

      case 'pri(signing.cancel)':
        return this.signingCancel(request as RequestSigningCancel);

      case 'pri(signing.isLocked)':
        return this.signingIsLocked(request as RequestSigningIsLocked);

      case 'pri(signing.requests)':
        return port && this.signingSubscribe(id, port);

      case 'pri(window.open)':
        return this.windowOpen(request as AllowedPath);

      default:
        throw new Error(`Unable to handle message of type ${type}`);
    }
  }
}
