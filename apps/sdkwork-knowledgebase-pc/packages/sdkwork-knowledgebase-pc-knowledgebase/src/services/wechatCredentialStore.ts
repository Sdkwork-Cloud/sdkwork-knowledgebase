import { isBlank } from '@sdkwork/utils';
import {
  invokeDesktopCommand,
  isTauriDesktopRuntime,
} from 'sdkwork-knowledgebase-pc-core/host';

const WECHAT_CREDENTIAL_NAMESPACE = 'sdkwork.knowledgebase.pc.wechat.credentials.v1';

type WechatCredentialPayload = {
  appSecret?: string;
  token?: string;
  encodingAesKey?: string;
  msgToken?: string;
  msgEncodingAESKey?: string;
};

export function isDesktopSecureStorageAvailable(): boolean {
  return isTauriDesktopRuntime();
}

function credentialKey(kind: 'official-account' | 'applet', id: string, field: keyof WechatCredentialPayload): string {
  return `${WECHAT_CREDENTIAL_NAMESPACE}.${kind}.${id}.${field}`;
}

async function readSecureValue(key: string): Promise<string | undefined> {
  if (!isTauriDesktopRuntime()) {
    return undefined;
  }
  try {
    const value = await invokeDesktopCommand<string | null>('read_secure_session_value', {
      request: { key },
    });
    return value && value.length > 0 ? value : undefined;
  } catch {
    return undefined;
  }
}

async function writeSecureValue(key: string, value: string | undefined): Promise<void> {
  if (!isTauriDesktopRuntime()) {
    return;
  }
  if (isBlank(value)) {
    await invokeDesktopCommand('remove_secure_session_value', { request: { key } }).catch(() => undefined);
    return;
  }
  await invokeDesktopCommand('write_secure_session_value', {
    request: { key, value: (value ?? '').trim() },
  }).catch(() => undefined);
}

export async function hydrateOfficialAccountSecrets<T extends { id: string; appSecret: string; token?: string; encodingAesKey?: string }>(
  account: T,
): Promise<T> {
  const [appSecret, token, encodingAesKey] = await Promise.all([
    readSecureValue(credentialKey('official-account', account.id, 'appSecret')),
    readSecureValue(credentialKey('official-account', account.id, 'token')),
    readSecureValue(credentialKey('official-account', account.id, 'encodingAesKey')),
  ]);
  return {
    ...account,
    appSecret: appSecret ?? account.appSecret ?? '',
    token: token ?? account.token,
    encodingAesKey: encodingAesKey ?? account.encodingAesKey,
  };
}

export async function persistOfficialAccountSecrets(account: {
  id: string;
  appSecret?: string;
  token?: string;
  encodingAesKey?: string;
}): Promise<void> {
  await Promise.all([
    writeSecureValue(credentialKey('official-account', account.id, 'appSecret'), account.appSecret),
    writeSecureValue(credentialKey('official-account', account.id, 'token'), account.token),
    writeSecureValue(credentialKey('official-account', account.id, 'encodingAesKey'), account.encodingAesKey),
  ]);
}

export async function hydrateAppletSecrets<T extends {
  id: string;
  appSecret?: string;
  msgToken?: string;
  msgEncodingAESKey?: string;
}>(applet: T): Promise<T> {
  const [appSecret, msgToken, msgEncodingAESKey] = await Promise.all([
    readSecureValue(credentialKey('applet', applet.id, 'appSecret')),
    readSecureValue(credentialKey('applet', applet.id, 'msgToken')),
    readSecureValue(credentialKey('applet', applet.id, 'msgEncodingAESKey')),
  ]);
  return {
    ...applet,
    appSecret: appSecret ?? applet.appSecret,
    msgToken: msgToken ?? applet.msgToken,
    msgEncodingAESKey: msgEncodingAESKey ?? applet.msgEncodingAESKey,
  };
}

export async function persistAppletSecrets(applet: {
  id: string;
  appSecret?: string;
  msgToken?: string;
  msgEncodingAESKey?: string;
}): Promise<void> {
  await Promise.all([
    writeSecureValue(credentialKey('applet', applet.id, 'appSecret'), applet.appSecret),
    writeSecureValue(credentialKey('applet', applet.id, 'msgToken'), applet.msgToken),
    writeSecureValue(credentialKey('applet', applet.id, 'msgEncodingAESKey'), applet.msgEncodingAESKey),
  ]);
}

export function stripOfficialAccountSecrets<T extends { appSecret?: string; token?: string; encodingAesKey?: string }>(
  account: T,
): T {
  return {
    ...account,
    appSecret: '',
    token: undefined,
    encodingAesKey: undefined,
  };
}

export function stripAppletSecrets<T extends { appSecret?: string; msgToken?: string; msgEncodingAESKey?: string }>(
  applet: T,
): T {
  return {
    ...applet,
    appSecret: undefined,
    msgToken: undefined,
    msgEncodingAESKey: undefined,
  };
}
