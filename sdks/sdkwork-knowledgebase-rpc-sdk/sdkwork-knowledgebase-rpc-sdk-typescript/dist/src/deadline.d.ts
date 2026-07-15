export interface RpcDeadlineOptions {
    timeoutMs?: number;
    deadline?: Date;
    now?: () => number;
}
export declare function resolveRpcDeadlineMs(options?: RpcDeadlineOptions): number | undefined;
