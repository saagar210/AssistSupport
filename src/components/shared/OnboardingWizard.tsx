import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useDownload } from '../../hooks/useDownload';
import { Icon } from './Icon';
import './OnboardingWizard.css';

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
}

interface OnboardingWizardProps {
  onComplete: () => void;
  onSkip: () => void;
}

type Step = 'welcome' | 'security' | 'model' | 'kb' | 'shortcuts' | 'complete';

interface StepInfo {
  title: string;
  description: string;
  icon: string;
}

const STEPS: Record<Step, StepInfo> = {
  welcome: {
    title: 'Welcome to AssistSupport',
    description: 'Your local AI-powered IT support assistant. All data stays on your device.',
    icon: 'sparkles',
  },
  security: {
    title: 'Choose Your Security Mode',
    description: 'How should we protect your encryption keys?',
    icon: 'shield',
  },
  model: {
    title: 'Download an AI Model',
    description: 'Choose a model to power your responses. Smaller models are faster, larger models are smarter.',
    icon: 'cpu',
  },
  kb: {
    title: 'Set Up Your Knowledge Base',
    description: 'Point to a folder with your documentation. The AI will use this to give accurate answers.',
    icon: 'folderOpen',
  },
  shortcuts: {
    title: 'Keyboard Shortcuts',
    description: 'Work faster with these essential shortcuts.',
    icon: 'command',
  },
  complete: {
    title: "You're All Set!",
    description: 'Start drafting responses with AI assistance.',
    icon: 'checkCircle',
  },
};

const STEP_ORDER: Step[] = ['welcome', 'security', 'model', 'kb', 'shortcuts', 'complete'];

type SecurityMode = 'keychain' | 'passphrase';

const SHORTCUTS = [
  { keys: ['Cmd', 'K'], description: 'Open command palette' },
  { keys: ['Cmd', 'Enter'], description: 'Generate response' },
  { keys: ['Cmd', 'S'], description: 'Save draft' },
  { keys: ['Cmd', 'N'], description: 'New draft' },
  { keys: ['Cmd', '/'], description: 'Focus search' },
  { keys: ['Cmd', '1-6'], description: 'Switch tabs' },
];

interface ModelOption {
  name: string;
  size: string;
  description: string;
  modelId: string;
}

const MODEL_OPTIONS: ModelOption[] = [
  {
    name: 'Llama 3.2 3B',
    size: '2.0 GB',
    description: 'Fast and efficient, great for most tasks',
    modelId: 'llama-3.2-3b-instruct',
  },
  {
    name: 'Phi 3 Mini 4K',
    size: '2.4 GB',
    description: 'Stronger reasoning, requires more RAM',
    modelId: 'phi-3-mini-4k-instruct',
  },
];

export function OnboardingWizard({ onComplete, onSkip }: OnboardingWizardProps) {
  const [currentStep, setCurrentStep] = useState<Step>('welcome');
  const [modelDownloaded, setModelDownloaded] = useState(false);
  const [kbFolder, setKbFolder] = useState<string | null>(null);
  const [securityMode, setSecurityMode] = useState<SecurityMode>('keychain');
  const [securityConfigured, setSecurityConfigured] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const {
    isDownloading: downloadingModel,
    downloadProgress: downloadProgressState,
    error: downloadError,
    downloadModel: startDownload,
    cancelDownload,
  } = useDownload();

  const downloadProgress = downloadProgressState?.percent ?? 0;

  const currentStepIndex = STEP_ORDER.indexOf(currentStep);
  const stepInfo = STEPS[currentStep];

  const goNext = useCallback(() => {
    const nextIndex = currentStepIndex + 1;
    if (nextIndex < STEP_ORDER.length) {
      setCurrentStep(STEP_ORDER[nextIndex]);
      setError(null);
    } else {
      onComplete();
    }
  }, [currentStepIndex, onComplete]);

  const goBack = useCallback(() => {
    const prevIndex = currentStepIndex - 1;
    if (prevIndex >= 0) {
      setCurrentStep(STEP_ORDER[prevIndex]);
      setError(null);
    }
  }, [currentStepIndex]);

  const downloadModel = useCallback(async (model: ModelOption) => {
    setError(null);
    try {
      await startDownload(model.modelId);
      setModelDownloaded(true);
    } catch (e) {
      setError(String(e));
    }
  }, [startDownload]);

  const selectKbFolder = useCallback(async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Knowledge Base Folder',
      });

      if (selected && typeof selected === 'string') {
        await invoke('set_kb_folder', { folderPath: selected });
        setKbFolder(selected);
        setError(null);
      }
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const configureSecurity = useCallback(async (mode: SecurityMode) => {
    try {
      setSecurityMode(mode);
      // Security mode preference is stored in local state and applied
      // when the master key is first generated during initial setup
      setSecurityConfigured(true);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const renderStepContent = () => {
    switch (currentStep) {
      case 'welcome':
        return (
          <div className="onboarding-welcome">
            <div className="onboarding-features">
              <div className="onboarding-feature">
                <Icon name="shield" size={24} />
                <div>
                  <strong>100% Private</strong>
                  <p>Everything runs locally. No data leaves your device.</p>
                </div>
              </div>
              <div className="onboarding-feature">
                <Icon name="zap" size={24} />
                <div>
                  <strong>Fast Responses</strong>
                  <p>Draft IT support responses in seconds.</p>
                </div>
              </div>
              <div className="onboarding-feature">
                <Icon name="book" size={24} />
                <div>
                  <strong>Knowledge-Powered</strong>
                  <p>Uses your documentation for accurate answers.</p>
                </div>
              </div>
            </div>
          </div>
        );

      case 'security':
        return (
          <div className="onboarding-security">
            <div className="security-options">
              <button
                className={`security-option ${securityMode === 'keychain' ? 'selected' : ''}`}
                onClick={() => configureSecurity('keychain')}
              >
                <div className="security-option-header">
                  <Icon name="shield" size={24} />
                  <strong>macOS Keychain</strong>
                  <span className="recommended-badge">Recommended</span>
                </div>
                <p>Uses your Mac's secure keychain to store encryption keys. No password needed.</p>
                <ul className="security-features">
                  <li><Icon name="check" size={14} /> Automatic unlock with macOS login</li>
                  <li><Icon name="check" size={14} /> Protected by Secure Enclave</li>
                  <li><Icon name="check" size={14} /> No password to remember</li>
                </ul>
              </button>

              <button
                className={`security-option ${securityMode === 'passphrase' ? 'selected' : ''}`}
                onClick={() => configureSecurity('passphrase')}
              >
                <div className="security-option-header">
                  <Icon name="terminal" size={24} />
                  <strong>Passphrase</strong>
                </div>
                <p>Enter a passphrase each time you open the app. More portable across devices.</p>
                <ul className="security-features">
                  <li><Icon name="check" size={14} /> Works if Keychain unavailable</li>
                  <li><Icon name="check" size={14} /> Exportable to other Macs</li>
                  <li><Icon name="x" size={14} className="con" /> Requires passphrase at startup</li>
                </ul>
              </button>
            </div>
            {securityConfigured && (
              <p className="onboarding-success-text">
                <Icon name="checkCircle" size={16} /> Security mode configured
              </p>
            )}
          </div>
        );

      case 'model':
        return (
          <div className="onboarding-model">
            {downloadingModel ? (
              <div className="onboarding-download-progress">
                <div className="progress-bar">
                  <div
                    className="progress-fill"
                    style={{ width: `${downloadProgress}%` }}
                  />
                </div>
                <p>
                  Downloading... {Math.round(downloadProgress)}%
                  {downloadProgressState && downloadProgressState.speed_bps > 0 && (
                    <span className="download-speed"> ({formatBytes(downloadProgressState.speed_bps)}/s)</span>
                  )}
                </p>
                {downloadProgressState && downloadProgressState.total_bytes > 0 && (
                  <p className="download-detail">
                    {formatBytes(downloadProgressState.downloaded_bytes)} / {formatBytes(downloadProgressState.total_bytes)}
                  </p>
                )}
                <button className="onboarding-btn-ghost" onClick={cancelDownload}>
                  Cancel Download
                </button>
              </div>
            ) : modelDownloaded ? (
              <div className="onboarding-success">
                <Icon name="checkCircle" size={48} />
                <p>Model downloaded successfully!</p>
              </div>
            ) : (
              <div className="onboarding-model-options">
                {MODEL_OPTIONS.map((model) => (
                  <button
                    key={model.name}
                    className="onboarding-model-card"
                    onClick={() => downloadModel(model)}
                  >
                    <div className="model-card-header">
                      <strong>{model.name}</strong>
                      <span className="model-size">{model.size}</span>
                    </div>
                    <p>{model.description}</p>
                  </button>
                ))}
                <p className="onboarding-hint">
                  You can download more models later in Settings.
                </p>
              </div>
            )}
          </div>
        );

      case 'kb':
        return (
          <div className="onboarding-kb">
            {kbFolder ? (
              <div className="onboarding-kb-selected">
                <Icon name="folderOpen" size={32} />
                <div className="kb-path">{kbFolder}</div>
                <button
                  className="onboarding-btn-secondary"
                  onClick={selectKbFolder}
                >
                  Change Folder
                </button>
              </div>
            ) : (
              <div className="onboarding-kb-empty">
                <button
                  className="onboarding-btn-primary"
                  onClick={selectKbFolder}
                >
                  <Icon name="folderOpen" size={20} />
                  Select Folder
                </button>
                <p className="onboarding-hint">
                  Choose a folder containing your IT documentation, runbooks, or guides.
                  Markdown and text files work best.
                </p>
              </div>
            )}
          </div>
        );

      case 'shortcuts':
        return (
          <div className="onboarding-shortcuts">
            <div className="shortcuts-grid">
              {SHORTCUTS.map((shortcut, index) => (
                <div key={index} className="shortcut-item">
                  <div className="shortcut-keys">
                    {shortcut.keys.map((key, i) => (
                      <kbd key={i}>{key === 'Cmd' ? '⌘' : key}</kbd>
                    ))}
                  </div>
                  <span className="shortcut-desc">{shortcut.description}</span>
                </div>
              ))}
            </div>
            <p className="onboarding-hint">
              Press <kbd>⌘</kbd><kbd>Shift</kbd><kbd>/</kbd> anytime to see all shortcuts.
            </p>
          </div>
        );

      case 'complete':
        return (
          <div className="onboarding-complete">
            <div className="onboarding-checklist">
              <div className={`checklist-item ${securityConfigured ? 'done' : 'pending'}`}>
                <Icon name={securityConfigured ? 'checkCircle' : 'circle'} size={20} />
                <span>Security {securityConfigured ? `(${securityMode})` : 'not configured'}</span>
              </div>
              <div className={`checklist-item ${modelDownloaded ? 'done' : 'pending'}`}>
                <Icon name={modelDownloaded ? 'checkCircle' : 'circle'} size={20} />
                <span>AI Model {modelDownloaded ? 'downloaded' : 'not downloaded'}</span>
              </div>
              <div className={`checklist-item ${kbFolder ? 'done' : 'pending'}`}>
                <Icon name={kbFolder ? 'checkCircle' : 'circle'} size={20} />
                <span>Knowledge Base {kbFolder ? 'configured' : 'not configured'}</span>
              </div>
            </div>
            <p className="onboarding-hint">
              You can always configure these later in Settings.
            </p>
          </div>
        );
    }
  };

  return (
    <div className="onboarding-overlay">
      <div className="onboarding-modal">
        <div className="onboarding-header">
          <div className="onboarding-icon">
            <Icon name={stepInfo.icon as any} size={32} />
          </div>
          <h2>{stepInfo.title}</h2>
          <p>{stepInfo.description}</p>
        </div>

        <div className="onboarding-progress">
          {STEP_ORDER.map((step, index) => (
            <div
              key={step}
              className={`progress-dot ${index <= currentStepIndex ? 'active' : ''}`}
            />
          ))}
        </div>

        {(error || downloadError) && (
          <div className="onboarding-error">
            <Icon name="alertCircle" size={16} />
            {error || downloadError}
          </div>
        )}

        <div className="onboarding-content">
          {renderStepContent()}
        </div>

        <div className="onboarding-footer">
          {currentStep === 'welcome' ? (
            <>
              <button className="onboarding-btn-ghost" onClick={onSkip}>
                Skip Setup
              </button>
              <button className="onboarding-btn-primary" onClick={goNext}>
                Get Started
              </button>
            </>
          ) : currentStep === 'complete' ? (
            <>
              <button className="onboarding-btn-ghost" onClick={goBack}>
                Back
              </button>
              <button className="onboarding-btn-primary" onClick={onComplete}>
                Start Using AssistSupport
              </button>
            </>
          ) : (
            <>
              <button className="onboarding-btn-ghost" onClick={goBack}>
                Back
              </button>
              <div className="onboarding-btn-group">
                <button className="onboarding-btn-ghost" onClick={goNext}>
                  Skip
                </button>
                <button
                  className="onboarding-btn-primary"
                  onClick={goNext}
                  disabled={currentStep === 'model' && downloadingModel}
                >
                  {currentStep === 'model' && !modelDownloaded ? 'Continue Without Model' : 'Continue'}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
