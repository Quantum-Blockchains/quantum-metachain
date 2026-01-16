// Copyright 2019-2023 @polkadot/extension-ui authors & contributors
// SPDX-License-Identifier: Apache-2.0

import type { ThemeProps } from '../../types.js';

import React, { useCallback, useContext, useEffect, useState } from 'react';

import { ActionContext, Button, ButtonArea, DidSigningReqContext, InputWithLabel, Link, Warning } from '../../components/index.js';
import useTranslation from '../../hooks/useTranslation.js';
import { Header } from '../../partials/index.js';
import { approveDidSignPassword, cancelDidSignRequest } from '../../messaging.js';
import { styled } from '../../styled.js';
import TransactionIndex from '../Signing/TransactionIndex.js';

interface Props extends ThemeProps {
  className?: string;
}

function DidSigning ({ className }: Props): React.ReactElement<Props> {
  const { t } = useTranslation();
  const requests = useContext(DidSigningReqContext);
  const [requestIndex, setRequestIndex] = useState(0);
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isBusy, setIsBusy] = useState(false);
  const onAction = useContext(ActionContext);

  const _onNextClick = useCallback(
    () => setRequestIndex((requestIndex) => requestIndex + 1),
    []
  );

  const _onPreviousClick = useCallback(
    () => setRequestIndex((requestIndex) => requestIndex - 1),
    []
  );

  useEffect(() => {
    setRequestIndex(
      (requestIndex) => requestIndex < requests.length
        ? requestIndex
        : requests.length - 1
    );
    setPassword('');
    setError(null);
  }, [requests]);

  const request = requests.length !== 0
    ? requestIndex >= 0
      ? requestIndex < requests.length
        ? requests[requestIndex]
        : requests[requests.length - 1]
      : requests[0]
    : null;

  const _onSign = useCallback((): void => {
    if (!request) {
      return;
    }

    setIsBusy(true);
    approveDidSignPassword(request.id, password)
      .then((): void => {
        setIsBusy(false);
        onAction();
      })
      .catch((error: Error): void => {
        setIsBusy(false);
        setError(error.message);
      });
  }, [onAction, password, request]);

  const _onCancel = useCallback((): void => {
    if (!request) {
      return;
    }

    cancelDidSignRequest(request.id)
      .then(() => onAction())
      .catch((error: Error) => console.error(error));
  }, [onAction, request]);

  if (!request) {
    return (
      <>
        <Header text={t<string>('DID signature')} />
      </>
    );
  }

  return (
    <>
      <Header text={t<string>('DID signature')}>
        {requests.length > 1 && (
          <TransactionIndex
            index={requestIndex}
            onNextClick={_onNextClick}
            onPreviousClick={_onPreviousClick}
            totalItems={requests.length}
          />
        )}
      </Header>
      <div className={className}>
        <div className='didCard'>
          <div className='didIcon'>ID</div>
          <div className='didBody'>
            <div className='didName'>
              {request.name || t<string>('Selected DID')}
            </div>
            <div className='didValue'>{request.did}</div>
            <div className='didOrigin'>
              {t<string>('Origin')}: {request.url}
            </div>
          </div>
        </div>
        <div className='didPassword'>
        <InputWithLabel
          isError={!password || !!error}
          label={t<string>('Password for this DID')}
          onChange={setPassword}
          onEnter={_onSign}
          type='password'
          value={password}
          withoutMargin
        />
        {error && (
          <Warning
            isBelowInput
            isDanger
          >
            {error}
          </Warning>
        )}
        </div>
      </div>
      <ButtonArea>
        <Button
          isBusy={isBusy}
          isDisabled={!password || !!error}
          onClick={_onSign}
        >
          {t<string>('Sign DID payload')}
        </Button>
      </ButtonArea>
      <ButtonArea>
        <Link
          isDanger
          onClick={_onCancel}
        >
          {t<string>('Cancel')}
        </Link>
      </ButtonArea>
    </>
  );
}

export default styled(DidSigning)(({ theme }: Props) => `
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 0 24px 8px;

  .didCard {
    align-items: center;
    background: ${theme.readonlyInputBackground};
    border: 1px solid ${theme.inputBorderColor};
    border-radius: ${theme.borderRadius};
    display: flex;
    gap: 12px;
    padding: 12px;
  }

  .didIcon {
    align-items: center;
    background: linear-gradient(135deg, #1f9d8f, #2a5d65);
    border-radius: 50%;
    color: #fff;
    display: flex;
    font-size: 12px;
    font-weight: 700;
    height: 36px;
    justify-content: center;
    letter-spacing: 0.4px;
    width: 36px;
  }

  .didBody {
    flex: 1;
    min-width: 0;
  }

  .didName {
    color: ${theme.textColor};
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 4px;
  }

  .didValue {
    color: ${theme.labelColor};
    font-size: 12px;
    word-break: break-all;
  }

  .didOrigin {
    color: ${theme.labelColor};
    font-size: 12px;
    margin-top: 6px;
  }

  .didPassword {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
`);
