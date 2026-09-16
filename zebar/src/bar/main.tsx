import React, { useEffect, useState } from 'react';
import { createRoot } from 'react-dom/client';
import * as zebar from 'zebar';
import type {
  CpuOutput,
  DateOutput,
  GlazeWmOutput,
  MemoryOutput,
  NetworkOutput,
  ProviderGroupConfig,
  SystrayOutput,
  WeatherOutput,
  WeatherStatus,
} from 'zebar';

import { Icon, type IconName } from '../shared/icons';
import { loadRuntimeConfig, type RuntimeConfig } from './runtime-config';
import './styles.css';

type BarOutput = Partial<{
  cpu: CpuOutput;
  date: DateOutput;
  glazewm: GlazeWmOutput;
  memory: MemoryOutput;
  network: NetworkOutput;
  systray: SystrayOutput;
  weather: WeatherOutput;
}>;

const pinnedTray = [
  /viscosity/i,
  /elgato|control\s*center/i,
  /windows\s*update|restart.*(required|pending)/i,
];

function shouldOwnTray(config: RuntimeConfig): boolean {
  if (config.monitor.trayMode === 'all') return true;
  if (config.monitor.trayMode === 'disabled') return false;
  return window.screenX === 0;
}

function networkIcon(output: NetworkOutput): IconName {
  if (output.defaultInterface?.type === 'ethernet') return 'ethernet';
  if (output.defaultInterface?.type === 'proprietary_virtual') return 'shield';
  if (output.defaultInterface?.type !== 'wifi') return 'wifiOff';
  const strength = output.defaultGateway?.signalStrength ?? 0;
  if (strength >= 65) return 'wifi0';
  if (strength >= 40) return 'wifi2';
  return strength >= 25 ? 'wifi1' : 'wifiOff';
}

function weatherIcon(status: WeatherStatus): IconName {
  if (status === 'clear_day') return 'sun';
  if (status === 'clear_night') return 'moon';
  if (status.includes('rain')) return 'rain';
  if (status.includes('snow')) return 'snow';
  if (status.includes('thunder')) return 'lightning';
  return 'cloud';
}

function App({ config }: { config: RuntimeConfig }) {
  const [providers] = useState(() => {
    const providerConfig: ProviderGroupConfig = {
      network: { type: 'network' },
      glazewm: { type: 'glazewm' },
      cpu: { type: 'cpu' },
      date: { type: 'date', formatting: config.clock.format },
      memory: { type: 'memory' },
      weather: { type: 'weather' },
    };
    if (shouldOwnTray(config)) providerConfig.systray = { type: 'systray' };
    return zebar.createProviderGroup(providerConfig);
  });
  const [output, setOutput] = useState<BarOutput>(
    providers.outputMap as BarOutput,
  );
  const [providerErrors, setProviderErrors] = useState<Record<string, string | null>>(
    providers.errorMap,
  );
  const [trayExpanded, setTrayExpanded] = useState(false);
  const [launcherState, setLauncherState] = useState<'idle' | 'pending' | 'error'>(
    'idle',
  );

  useEffect(() => {
    providers.onOutput(next => setOutput(next as BarOutput));
    providers.onError(next => setProviderErrors({ ...next }));
  }, [providers]);

  async function openLauncher() {
    if (launcherState === 'pending' || !config.launcher.program) return;
    setLauncherState('pending');
    try {
      const result = await zebar.shellExec(config.launcher.program, []);
      if (result.code !== 0) throw new Error(`launcher exit ${result.code}`);
      setLauncherState('idle');
    } catch {
      setLauncherState('error');
      window.setTimeout(() => setLauncherState('idle'), 3000);
    }
  }

  const clockUnavailable = !output.date || Boolean(providerErrors.date);
  const weatherUnavailable = !output.weather || Boolean(providerErrors.weather);
  const weather = weatherUnavailable ? undefined : output.weather;

  return (
    <div className="app">
      <div className="left">
        <span
          className="health-dot health-unknown"
          title="Winmakase health unavailable until the shared status source lands"
        />
        <button
          className={`launcher-button launcher-${launcherState}`}
          title={
            !config.launcher.program
              ? 'PowerToys Run helper is not configured'
              : launcherState === 'error'
                ? 'PowerToys Run could not open'
                : 'Open or focus PowerToys Run'
          }
          aria-label="Open or focus PowerToys Run"
          disabled={launcherState === 'pending' || !config.launcher.program}
          onClick={openLauncher}
        >
          <Icon name="magnify" />
        </button>
        {output.glazewm && (
          <div className="workspaces">
            {output.glazewm.currentWorkspaces
              .filter(workspace => workspace.name !== 'scratch')
              .map(workspace => (
                <button
                  className={`workspace ${workspace.hasFocus ? 'focused' : ''} ${workspace.isDisplayed ? 'displayed' : ''}`}
                  onClick={() =>
                    output.glazewm?.runCommand(
                      `focus --workspace ${workspace.name}`,
                    )
                  }
                  key={workspace.name}
                >
                  {workspace.displayName ?? workspace.name}
                </button>
              ))}
          </div>
        )}
      </div>

      <div className="center" title={clockUnavailable ? 'Clock unavailable' : undefined}>
        {clockUnavailable ? 'Time unavailable' : output.date?.formatted}
      </div>

      <div className="right">
        {output.glazewm && (
          <>
            {output.glazewm.isPaused && (
              <button
                className="paused-button"
                onClick={() => output.glazewm?.runCommand('wm-toggle-pause')}
              >
                PAUSED
              </button>
            )}
            {output.glazewm.bindingModes.map(bindingMode => (
              <button
                className="binding-mode"
                key={bindingMode.name}
                onClick={() =>
                  output.glazewm?.runCommand(
                    `wm-disable-binding-mode --name ${bindingMode.name}`,
                  )
                }
              >
                {bindingMode.displayName ?? bindingMode.name}
              </button>
            ))}
            <button
              className="tiling-direction"
              aria-label="Toggle tiling direction"
              onClick={() =>
                output.glazewm?.runCommand('toggle-tiling-direction')
              }
            >
              <Icon
                name={
                  output.glazewm.tilingDirection === 'horizontal'
                    ? 'swapHorizontal'
                    : 'swapVertical'
                }
              />
            </button>
          </>
        )}

        {output.network && (
          <div className="metric network">
            <Icon name={networkIcon(output.network)} />
            {output.network.defaultGateway?.ssid}
          </div>
        )}
        {output.memory && (
          <div className="metric memory">
            <Icon name="chip" />
            {Math.round(output.memory.usage)}%
          </div>
        )}
        {output.cpu && (
          <div className="metric cpu">
            <Icon name="chip" />
            <span className={output.cpu.usage > 85 ? 'high-usage' : ''}>
              {Math.round(output.cpu.usage)}%
            </span>
          </div>
        )}
        {!weather ? (
          <div className="metric weather unavailable" title="Weather unavailable">
            <Icon name="cloud" />—
          </div>
        ) : (
          <div className="metric weather">
            <Icon name={weatherIcon(weather.status)} />
            {Math.round(weather.celsiusTemp)}°C
          </div>
        )}

        {output.systray && (
          <div className="systray">
            {output.systray.icons
              .filter(
                icon =>
                  trayExpanded ||
                  pinnedTray.some(pattern => pattern.test(icon.tooltip ?? '')),
              )
              .map(icon => (
                <img
                  key={icon.id}
                  className="systray-icon"
                  src={icon.iconUrl}
                  title={icon.tooltip ?? undefined}
                  alt=""
                  onClick={event => {
                    event.preventDefault();
                    void output.systray?.onLeftClick(icon.id);
                  }}
                  onContextMenu={event => {
                    event.preventDefault();
                    void output.systray?.onRightClick(icon.id);
                  }}
                />
              ))}
            <button
              className="tray-expander"
              title={trayExpanded ? 'Collapse tray' : 'Show all tray icons'}
              aria-label={trayExpanded ? 'Collapse tray' : 'Show all tray icons'}
              onClick={() => setTrayExpanded(expanded => !expanded)}
            >
              {trayExpanded ? '›' : '‹'}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

async function start() {
  const config = await loadRuntimeConfig();
  const root = document.getElementById('root');
  if (!root) throw new Error('Missing #root element');
  createRoot(root).render(<App config={config} />);
}

void start();
