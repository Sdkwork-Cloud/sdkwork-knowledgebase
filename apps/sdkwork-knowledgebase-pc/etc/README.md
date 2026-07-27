# Knowledgebase PC Source Configuration

This application root delegates deployment profile ownership to
`../../../etc/sdkwork.deployment.config.json`. Run
`pnpm workflow:materialize-client-env` from the repository root to regenerate
the tracked `.env.<deploymentProfile>.<environment>` files. Local overrides use
ignored `.local` or `.bootstrap.local` files and must not contain committed
credentials.
