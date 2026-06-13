# SDKWork Knowledgebase Agent Provider

Thin adapter from SDKWork Knowledgebase retrieval contracts to `sdkwork-agent-kernel::KnowledgeProvider`.

The crate does not own vector-store logic, HTTP transport, generated SDK output, model invocation, memory, or tool execution. Consumers inject a typed `KnowledgebaseRetrievalClient` backed by the generated SDK, local service port, or service runtime adapter.
