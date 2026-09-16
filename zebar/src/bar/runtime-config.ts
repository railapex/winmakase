export interface RuntimeConfig {
  schemaVersion: 1;
  launcher: {
    program: string;
  };
  monitor: {
    trayMode: 'primary-at-origin' | 'all' | 'disabled';
  };
  clock: {
    format: string;
  };
}

const fallback: RuntimeConfig = {
  schemaVersion: 1,
  launcher: { program: '' },
  monitor: { trayMode: 'disabled' },
  clock: { format: 'EEE d MMM HH:mm' },
};

function isRuntimeConfig(value: unknown): value is RuntimeConfig {
  if (!value || typeof value !== 'object') return false;
  const config = value as Partial<RuntimeConfig>;
  return (
    config.schemaVersion === 1 &&
    typeof config.launcher?.program === 'string' &&
    ['primary-at-origin', 'all', 'disabled'].includes(
      config.monitor?.trayMode ?? '',
    ) &&
    typeof config.clock?.format === 'string'
  );
}

export async function loadRuntimeConfig(): Promise<RuntimeConfig> {
  try {
    const response = await fetch('./assets/runtime-config.json', {
      cache: 'no-store',
    });
    if (!response.ok) return fallback;
    const value: unknown = await response.json();
    return isRuntimeConfig(value) ? value : fallback;
  } catch {
    return fallback;
  }
}
