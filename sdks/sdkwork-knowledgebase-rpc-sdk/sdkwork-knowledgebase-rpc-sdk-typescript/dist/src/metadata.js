export function createStaticMetadataProvider(metadata) {
    return () => ({ ...metadata });
}
export function mergeRpcMetadata(...metadata) {
    const merged = {};
    for (const item of metadata) {
        if (!item) {
            continue;
        }
        for (const [key, value] of Object.entries(item)) {
            if (value !== undefined && value !== null && value !== '') {
                merged[key] = value;
            }
        }
    }
    return merged;
}
