import type { GenFile, GenMessage } from "@bufbuild/protobuf/codegenv2";
import type { Message } from "@bufbuild/protobuf";
/**
 * Describes the file sdkwork/common/v1/context.proto.
 */
export declare const file_sdkwork_common_v1_context: GenFile;
/**
 * Typed correlation metadata mirrors framework-verified metadata for generated clients. It is
 * never an authority source: internal service identity and caller scope come only from mTLS and
 * the signed framework caller context.
 *
 * @generated from message sdkwork.common.v1.RequestMetadata
 */
export type RequestMetadata = Message<"sdkwork.common.v1.RequestMetadata"> & {
    /**
     * @generated from field: string trace_id = 1;
     */
    traceId: string;
    /**
     * @generated from field: string traceparent = 2;
     */
    traceparent: string;
    /**
     * @generated from field: string idempotency_key = 3;
     */
    idempotencyKey: string;
    /**
     * @generated from field: string request_hash = 4;
     */
    requestHash: string;
    /**
     * @generated from field: string client_version = 5;
     */
    clientVersion: string;
};
/**
 * Describes the message sdkwork.common.v1.RequestMetadata.
 * Use `create(RequestMetadataSchema)` to create a new message.
 */
export declare const RequestMetadataSchema: GenMessage<RequestMetadata>;
/**
 * @generated from message sdkwork.common.v1.ResponseMetadata
 */
export type ResponseMetadata = Message<"sdkwork.common.v1.ResponseMetadata"> & {
    /**
     * @generated from field: string trace_id = 1;
     */
    traceId: string;
    /**
     * @generated from field: string traceparent = 2;
     */
    traceparent: string;
    /**
     * @generated from field: string server_time = 3;
     */
    serverTime: string;
    /**
     * @generated from field: repeated string warnings = 4;
     */
    warnings: string[];
    /**
     * @generated from field: repeated string deprecation_notices = 5;
     */
    deprecationNotices: string[];
};
/**
 * Describes the message sdkwork.common.v1.ResponseMetadata.
 * Use `create(ResponseMetadataSchema)` to create a new message.
 */
export declare const ResponseMetadataSchema: GenMessage<ResponseMetadata>;
