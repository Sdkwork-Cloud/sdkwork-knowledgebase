export interface KnowledgeWechatApplet {
  id: string;
  name: string;
  appId: string;
  originalId?: string;
  appSecret?: string;
  path: string;
  avatar: string;
  group?: string;
  description?: string;
  requestDomain?: string[];
  socketDomain?: string[];
  uploadDomain?: string[];
  downloadDomain?: string[];
  udpDomain?: string[];
  tcpDomain?: string[];
  businessDomain?: string[];
  domainVerifyFileName?: string;
  /** WeChat domain verification text. The service also enforces a 65,536 UTF-8 byte limit. */
  domainVerifyFileContent?: string;
  msgToken?: string;
  msgEncodingAESKey?: string;
  msgDataFormat?: 'json' | 'xml';
  msgEncryptMode?: 'plain' | 'compatible' | 'safe';
}
