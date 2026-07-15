export type RpcMetadata = Record<string, string>;
export interface RpcMetadataProviderContext {
    service?: string;
    method?: string;
    operationId?: string;
}
export type RpcMetadataProvider = (context: RpcMetadataProviderContext) => RpcMetadata | Promise<RpcMetadata>;
export declare function createStaticMetadataProvider(metadata: RpcMetadata): RpcMetadataProvider;
export declare function mergeRpcMetadata(...metadata: Array<RpcMetadata | undefined>): RpcMetadata;
