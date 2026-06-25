import type { DesktopPlatform } from './settingsDesktopBridge';

export function resolveAutoStartDescriptionKey(platform: DesktopPlatform | null): string {
  switch (platform) {
    case 'windows':
      return 'autoStartDescriptionWindows';
    case 'macos':
      return 'autoStartDescriptionMacos';
    case 'linux':
      return 'autoStartDescriptionLinux';
    default:
      return 'autoStartDescription';
  }
}

export function resolveHideToTrayDescriptionKey(platform: DesktopPlatform | null): string {
  switch (platform) {
    case 'windows':
      return 'hideToTrayDescriptionWindows';
    case 'macos':
      return 'hideToTrayDescriptionMacos';
    case 'linux':
      return 'hideToTrayDescriptionLinux';
    default:
      return 'hideToTrayDescription';
  }
}

export function resolveNativePlatformLabel(
  platform: DesktopPlatform | null,
  translate: (key: string) => string,
): string {
  switch (platform) {
    case 'windows':
      return translate('desktopPlatformWindows');
    case 'macos':
      return translate('desktopPlatformMacos');
    case 'linux':
      return translate('desktopPlatformLinux');
    default:
      return translate('desktopPlatformUnknown');
  }
}
