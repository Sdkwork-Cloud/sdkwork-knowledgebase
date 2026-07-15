# SDKWork Knowledgebase IM RPC Adapter Specs

This directory owns the component contract for the trusted Knowledgebase-to-IM RPC adapter.

- Machine contract: `component.spec.json`
- Public surface: typed mTLS configuration and the generated-client implementation of `GroupLaunchTicketConsumer`
- Runtime environment parsing remains owned by the standalone gateway bootstrap.

The adapter consumes the generated IM RPC SDK and SDKWork RPC framework. It does not own an HTTP
authority, an RPC server, persistence, or caller authentication.
