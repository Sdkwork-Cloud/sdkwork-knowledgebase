> Owner: SDKWork maintainers  
> Status: **completed** — RAG contracts, agent profile bindings, service ports, and kernel adapter foundations are implemented.

**Goal:** Add standard SDKWork Knowledgebase RAG, knowledge-agent profile, API, database, and agent-kernel adapter foundations.

**Outcome:** RAG contract types, hybrid retrieval/context-pack services, agent profile storage, `sdkwork-knowledgebase-agent-provider` adapter, and integration tests.

**Verification:**

```bash
cargo test -p sdkwork-knowledgebase-contract rag
cargo test -p sdkwork-intelligence-knowledgebase-service knowledge_engine
pnpm check:knowledge-engine-spi
```

**Design reference:** [TECH-2026-06-09-knowledgebase-agent-rag-design.md](TECH-2026-06-09-knowledgebase-agent-rag-design.md)
