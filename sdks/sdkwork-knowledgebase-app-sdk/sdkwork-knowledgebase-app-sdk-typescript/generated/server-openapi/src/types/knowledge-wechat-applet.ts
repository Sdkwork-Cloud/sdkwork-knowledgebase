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
  domainVerifyFileContent?: string;
  msgToken?: string;
  msgEncodingAESKey?: string;
  msgDataFormat?: string;
  msgEncryptMode?: string;
}
