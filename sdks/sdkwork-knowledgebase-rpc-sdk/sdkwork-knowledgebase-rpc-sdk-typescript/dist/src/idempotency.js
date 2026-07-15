export function createRpcIdempotencyMetadata(options = {}) {
    const metadata = {};
    if (options.idempotencyKey) {
        metadata['idempotency-key'] = options.idempotencyKey;
    }
    if (options.requestHash) {
        metadata['x-request-hash'] = options.requestHash;
    }
    return metadata;
}
