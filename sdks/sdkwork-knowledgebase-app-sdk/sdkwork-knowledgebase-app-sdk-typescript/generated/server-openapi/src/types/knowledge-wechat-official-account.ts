export interface KnowledgeWechatOfficialAccount {
  id: string;
  name: string;
  type: string;
  avatar: string;
  description?: string;
  appId: string;
  appSecret?: string;
  serverUrl?: string;
  token?: string;
  encodingAesKey?: string;
  encryptMode?: string;
  domainVerifyFileName?: string;
  domainVerifyFileContent?: string;
  jsSecureDomains?: string[];
  webAuthDomains?: string[];
  businessDomains?: string[];
  group?: string;
}
