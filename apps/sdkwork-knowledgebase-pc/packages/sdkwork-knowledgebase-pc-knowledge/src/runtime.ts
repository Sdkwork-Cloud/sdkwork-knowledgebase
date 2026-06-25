import type { KnowledgebasePcSdkPorts } from './sdkPorts';
import { configureKnowledgebasePcSdkPorts } from './sdkPorts';

export interface ConfigureKnowledgebasePcRuntimeOptions {
  sdkPorts: KnowledgebasePcSdkPorts;
}

export function configureKnowledgebasePcRuntime(options: ConfigureKnowledgebasePcRuntimeOptions): void {
  configureKnowledgebasePcSdkPorts(options.sdkPorts);
}

export { configureKnowledgebasePcRuntime as configureKnowledgebasePcRuntimeFromKnowledgebasePackage };
