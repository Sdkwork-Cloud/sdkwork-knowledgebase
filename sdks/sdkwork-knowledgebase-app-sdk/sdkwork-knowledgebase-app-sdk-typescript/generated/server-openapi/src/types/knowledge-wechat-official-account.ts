export interface KnowledgeWechatOfficialAccount {
  id: string;
  name: string;
  type: 'subscription' | 'service';
  avatar: string;
  description?: string;
  appId: string;
  appSecret?: string;
  serverUrl?: string;
  token?: string;
  encodingAesKey?: string;
  encryptMode?: 'plain' | 'compatible' | 'safe';
  domainVerifyFileName?: string;
  /** WeChat domain verification text. The service also enforces a 65,536 UTF-8 byte limit. */
  domainVerifyFileContent?: string;
  jsSecureDomains?: string[];
  webAuthDomains?: string[];
  businessDomains?: string[];
  group?: string;
}
