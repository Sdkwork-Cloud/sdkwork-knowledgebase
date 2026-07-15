export interface RpcIdempotencyOptions {
    idempotencyKey?: string;
    requestHash?: string;
}
export declare function createRpcIdempotencyMetadata(options?: RpcIdempotencyOptions): Record<string, string>;
