/**
 * SdkWork HTTP response envelope helpers for OpenAPI authority patches.
 * Authority: sdkwork-specs/API_SPEC.md section 4.5 and section 15.
 */

export function resourceEnvelope(itemSchemaRef) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: {
            type: 'object',
            required: ['item'],
            properties: {
              item: { $ref: itemSchemaRef },
            },
          },
        },
      },
    ],
  };
}

export function listEnvelope(itemSchemaRef) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: {
            type: 'object',
            required: ['items', 'pageInfo'],
            properties: {
              items: {
                type: 'array',
                items: { $ref: itemSchemaRef },
              },
              pageInfo: { $ref: '#/components/schemas/PageInfo' },
            },
          },
        },
      },
    ],
  };
}

export function listDataEnvelope(dataSchemaRef) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: { $ref: dataSchemaRef },
        },
      },
    ],
  };
}

export function browserListEnvelope(dataSchemaRef = '#/components/schemas/KnowledgeBrowserListData') {
  return listDataEnvelope(dataSchemaRef);
}

export function commandEnvelope(payloadSchemaRef) {
  return {
    allOf: [
      { $ref: '#/components/schemas/SdkWorkApiResponse' },
      {
        type: 'object',
        required: ['data'],
        properties: {
          data: { $ref: payloadSchemaRef },
        },
      },
    ],
  };
}

export const listPaginationQueryParams = [
  {
    name: 'cursor',
    in: 'query',
    required: false,
    schema: { type: 'string' },
  },
  {
    name: 'page_size',
    in: 'query',
    required: false,
    schema: {
      type: 'integer',
      format: 'int32',
      minimum: 1,
      maximum: 200,
    },
  },
];

export function jsonResponse(schema) {
  return {
    description: 'OK',
    content: {
      'application/json': { schema },
    },
  };
}

export function createdResponse(schema) {
  return {
    description: 'Created',
    content: {
      'application/json': { schema },
    },
  };
}
