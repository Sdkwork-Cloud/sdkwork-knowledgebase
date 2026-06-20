Upstream git submodule pins for external knowledge engines.

Submodules are **optional**. The authoritative registry is `../catalog.manifest.json`.

Initialize a vendor pin when adapter development needs local upstream source:

```bash
git submodule add -b main https://github.com/langgenius/dify.git external/knowledge-engines/upstream/dify
```

See `../README.md` and `node ../../tools/sync_external_knowledge_engine_submodules.mjs --check`.
